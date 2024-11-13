use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        bump_farm_index, extend_instance_ttl, get_block, get_farm_asset, get_farm_entropy,
        get_farm_index, get_farm_paused, get_pail, has_pail, remove_pail, set_block,
        set_farm_entropy, set_pail,
    },
    types::Block,
    Contract, ContractClient, FarmTrait, BLOCK_INTERVAL, BLOCK_REWARD, ZEROS_EXPONENT,
};

#[contractimpl]
impl FarmTrait for Contract {
    fn plant(env: Env, farmer: Address, amount: i128) {
        farmer.require_auth();

        let asset = get_farm_asset(&env);
        let mut index = get_farm_index(&env);
        let entropy = get_farm_entropy(&env);
        let paused = get_farm_paused(&env);

        if paused {
            panic_with_error!(&env, &Errors::FarmIsPaused);
        }

        if amount < 0 {
            panic_with_error!(&env, &Errors::PailAmountTooLow);
        }

        let mut block = match get_block(&env, index) {
            // genesis or evicted
            None => Block {
                timestamp: env.ledger().timestamp(),
                entropy,
                staked: 0,
                reclaimed: 0,
                pow_zeros: 0,
            },
            Some(mut block) => {
                // if the block is >= 1 minute old, we need to create a new one
                if env.ledger().timestamp() >= block.timestamp + BLOCK_INTERVAL {
                    block = Block {
                        timestamp: env.ledger().timestamp(),
                        entropy,
                        staked: 0,
                        reclaimed: 0,
                        pow_zeros: 0,
                    };

                    index = bump_farm_index(&env, index);
                }

                block
            }
        };

        // must come after block discovery as the index may have been bumped
        if has_pail(&env, farmer.clone(), index) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        block.staked += amount as u64;

        if amount > 0 {
            token::Client::new(&env, &asset).transfer(&farmer, &asset, &amount);
        }

        // NOTE consider adding a zero_count commitment to the pail vs just a stake amount
        // This would ensure folks couldn't run a lot of initial `work`'s for low zero counts as they tried to find a highest
        // I think initially though I want to try this version and see what happens

        set_pail(&env, farmer.clone(), index, amount, None);
        set_block(&env, index, &block);

        extend_instance_ttl(&env);
    }

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u128) {
        // No auth_require here so others can call this function on the `farmer`'s behalf

        let index = get_farm_index(&env);
        let mut block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let (pail, kale) = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));
        let generated_hash = generate_hash(&env, &index, &nonce, &block.entropy, &farmer);

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::HashIsInvalid);
        }

        let mut zero_count = 0;

        for byte in hash {
            if byte == 0 {
                zero_count += 2;
            } else {
                zero_count += byte.leading_zeros() / 4;
                break;
            }
        }

        match kale {
            Some(prev_zero_count) => {
                if zero_count <= prev_zero_count {
                    panic_with_error!(&env, &Errors::ZeroCountTooLow);
                }

                block.pow_zeros = block.pow_zeros + (ZEROS_EXPONENT.pow(zero_count) * pail)
                    - (ZEROS_EXPONENT.pow(prev_zero_count) * pail);
            }
            None => {
                block.pow_zeros = block.pow_zeros + (ZEROS_EXPONENT.pow(zero_count) * pail);
                block.reclaimed += pail as u64;
            }
        }

        set_pail(&env, farmer, index, pail, Some(zero_count));
        set_block(&env, index, &block);
        set_farm_entropy(&env, &generated_hash);

        extend_instance_ttl(&env);
    }

    fn harvest(env: Env, farmer: Address, index: u32) -> i128 {
        let asset = get_farm_asset(&env);
        let farm_index = get_farm_index(&env);
        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let (pail, kale) = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        if index >= farm_index {
            panic_with_error!(&env, &Errors::HarvestNotReady);
        }

        if kale.is_none() {
            panic_with_error!(&env, &Errors::KaleNotFound);
        }

        let full_block_reward = BLOCK_REWARD + block.staked;
        let actual_block_reward = full_block_reward - block.reclaimed;

        let kale = kale.unwrap();
        let reward = (ZEROS_EXPONENT.pow(kale) * pail).fixed_div_floor(
            &env,
            &(block.pow_zeros),
            &(actual_block_reward as i128),
        ) + pail;

        token::StellarAssetClient::new(&env, &asset).mint(&farmer, &reward);

        remove_pail(&env, farmer.clone(), index);

        extend_instance_ttl(&env);

        reward
    }
}

fn generate_hash(
    env: &Env,
    index: &u32,
    nonce: &u128,
    entropy: &BytesN<32>,
    farmer: &Address,
) -> BytesN<32> {
    let mut hash_array = [0u8; 84];

    let mut farmer_array = [0u8; 32];
    let farmer_bytes = farmer.to_xdr(env);
    farmer_bytes
        .slice(farmer_bytes.len() - 32..)
        .copy_into_slice(&mut farmer_array);

    hash_array[..4].copy_from_slice(&index.to_be_bytes());
    hash_array[4..4 + 16].copy_from_slice(&nonce.to_be_bytes());
    hash_array[20..20 + 32].copy_from_slice(&entropy.to_array());
    hash_array[52..].copy_from_slice(&farmer_array);

    env.crypto()
        .keccak256(&Bytes::from_array(env, &hash_array))
        .to_bytes()
}
