use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        bump_farm_index, extend_instance_ttl, get_block, get_farm_asset, get_farm_entropy,
        get_farm_index, get_farm_paused, get_pail, has_pail, remove_pail, set_block,
        set_farm_entropy, set_pail,
    },
    types::{Block, Pail},
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
        let mut block = match get_block(&env, index) {
            // genesis or evicted
            None => generate_block(&env, entropy),
            Some(mut block) => {
                // if the block is >= 1 minute old, we need to create a new one
                if env.ledger().timestamp() >= block.timestamp + BLOCK_INTERVAL {
                    block = generate_block(&env, entropy);
                    index = bump_farm_index(&env, index);
                }

                block
            }
        };

        if paused {
            panic_with_error!(&env, &Errors::FarmIsPaused);
        }

        if amount < 0 {
            panic_with_error!(&env, &Errors::PlantAmountTooLow);
        }

        // NOTE must come after block discovery as the index may have been bumped
        if has_pail(&env, farmer.clone(), index) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        block.staked += amount as u64;

        // NOTE: we allow passing zeros but zeros further down the stack will cause issues
        // So either A) we should enforce requiring a > 0 value
        // or B) set the min value to 1 (which will cause the interesting side affect of being able to "free" mint 1 stroop of value)

        if amount > 0 {
            token::Client::new(&env, &asset).burn(&farmer, &amount);
        }

        let pail = Pail {
            plant_seq: env.ledger().sequence(),
            work_seq: None,
            stake: amount.max(1), // ensure stake is at least 1 so new farmers will get _something_
            zeros: None,
        };

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);

        extend_instance_ttl(&env);
    }

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u128) {
        // No auth_require here so others can call this function on the `farmer`'s behalf

        let index = get_farm_index(&env);
        let mut block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let mut pail = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));
        let generated_hash = generate_hash(&env, &index, &nonce, &block.entropy, &farmer);

        let sequence = env.ledger().sequence();
        let mut zeros = 0;

        // Ensure there's been at least one ledger since
        if sequence <= pail.plant_seq {
            panic_with_error!(&env, &Errors::WorkNotReady);
        }

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::HashIsInvalid);
        }

        for byte in hash {
            if byte == 0 {
                zeros += 2;
            } else {
                zeros += byte.leading_zeros() / 4;
                break;
            }
        }

        block.pow_zeros = block.pow_zeros + (ZEROS_EXPONENT.pow(zeros) * pail.stake * (sequence - pail.plant_seq) as i128);

        match pail.zeros {
            Some(pail_zeros) => {
                if zeros <= pail_zeros {
                    panic_with_error!(&env, &Errors::ZeroCountTooLow);
                }

                block.pow_zeros = block.pow_zeros - (ZEROS_EXPONENT.pow(pail_zeros) * pail.stake);
            }
            None => {
                block.reclaimed += pail.stake as u64;
            }
        }

        pail.work_seq = Some(sequence);
        pail.zeros = Some(zeros);

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);
        set_farm_entropy(&env, &generated_hash);

        extend_instance_ttl(&env);
    }

    fn harvest(env: Env, farmer: Address, index: u32) -> i128 {
        let asset = get_farm_asset(&env);
        let farm_index = get_farm_index(&env);
        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let Pail { plant_seq, work_seq, stake, zeros } = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        if index >= farm_index {
            panic_with_error!(&env, &Errors::HarvestNotReady);
        }

        if work_seq.is_none() || zeros.is_none() {
            panic_with_error!(&env, &Errors::WorkNotFound);
        }

        let full_block_reward = BLOCK_REWARD + block.staked;
        let actual_block_reward = full_block_reward - block.reclaimed;

        let work_seq = work_seq.unwrap();
        let zeros = zeros.unwrap();

        let reward = (ZEROS_EXPONENT.pow(zeros) * stake * (work_seq - plant_seq) as i128).fixed_div_floor(
            &env,
            &(block.pow_zeros),
            &(actual_block_reward as i128),
        ) + stake;

        token::StellarAssetClient::new(&env, &asset).mint(&farmer, &reward);

        remove_pail(&env, farmer.clone(), index);

        extend_instance_ttl(&env);

        reward
    }
}

fn generate_block(env: &Env, entropy: BytesN<32>) -> Block {
    Block {
        timestamp: env.ledger().timestamp(),
        entropy,
        staked: 0,
        reclaimed: 0,
        pow_zeros: 0,
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
