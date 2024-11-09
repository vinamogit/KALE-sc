use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        bump_mine_index, extend_instance_ttl, get_block, get_mine_asset, get_mine_entropy,
        get_mine_index, get_mine_paused, get_pail, has_pail, remove_pail, set_block,
        set_mine_entropy, set_pail,
    },
    types::Block,
    KalepailTrait, MineKalepailContract, MineKalepailContractClient, BLOCK_REWARD, MINER_EXPONENT,
};

#[contractimpl]
impl KalepailTrait for MineKalepailContract {
    fn get_pail(env: Env, miner: Address, amount: i128) {
        miner.require_auth();

        let asset = get_mine_asset(&env);
        let mut index = get_mine_index(&env);
        let entropy = get_mine_entropy(&env);
        let paused = get_mine_paused(&env);

        if paused {
            panic_with_error!(&env, &Errors::MineIsPaused);
        }

        if amount < 0 {
            panic_with_error!(&env, &Errors::PailAmountTooLow);
        }

        let mut block = match get_block(&env, index) {
            // genesis or evicted
            None => Block {
                timestamp: env.ledger().timestamp(),
                entropy,
                pool: 0,
                claimed_pool: 0,
                pow_zeros: 0,
            },
            Some(mut block) => {
                // if the block is >= 1 minute old, we need to create a new one
                if env.ledger().timestamp() >= block.timestamp + 60 {
                    block = Block {
                        timestamp: env.ledger().timestamp(),
                        entropy,
                        pool: 0,
                        claimed_pool: 0,
                        pow_zeros: 0,
                    };

                    index = bump_mine_index(&env, index);
                }

                block
            }
        };

        // must come after block discovery as the index may have been bumped
        if has_pail(&env, miner.clone(), index) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        block.pool += amount as u64;

        if amount > 0 {
            token::Client::new(&env, &asset).transfer(&miner, &asset, &amount);
        }

        // NOTE consider adding a zero_count commitment to the pail vs just a stake amount
        // This would ensure folks couldn't run a lot of initial get_kale's for low zero counts as they tried to find a highest
        // I think initially though I want to try this version and see what happens

        set_pail(&env, miner.clone(), index, amount, None);
        set_block(&env, index, &block);

        extend_instance_ttl(&env);
    }

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128) {
        // No auth_require here so others can call this function on the `miner`'s behalf

        let index = get_mine_index(&env);
        let mut block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let (pail, kale) = get_pail(&env, miner.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));
        let generated_hash = generate_hash(&env, &miner, &index, &nonce, &block.entropy);

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

                block.pow_zeros = block.pow_zeros + (MINER_EXPONENT.pow(zero_count) * pail as u64)
                    - (MINER_EXPONENT.pow(prev_zero_count) * pail as u64);
            }
            None => {
                block.pow_zeros = block.pow_zeros + (MINER_EXPONENT.pow(zero_count) * pail as u64);
                block.claimed_pool += pail as u64;
            }
        }

        set_pail(&env, miner, index, pail, Some(zero_count));
        set_block(&env, index, &block);
        set_mine_entropy(&env, &generated_hash);

        extend_instance_ttl(&env);
    }

    fn claim_kale(env: Env, miner: Address, index: u32) {
        let asset = get_mine_asset(&env);
        let mine_index = get_mine_index(&env);
        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));
        let (pail, kale) = get_pail(&env, miner.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        if index >= mine_index {
            panic_with_error!(&env, &Errors::TooSoonToClaim);
        }

        if kale.is_none() {
            panic_with_error!(&env, &Errors::KaleNotFound);
        }

        let full_block_reward = BLOCK_REWARD + block.pool;
        let actual_block_reward = (full_block_reward - block.claimed_pool) as i128;

        let kale = kale.unwrap();
        let reward = (MINER_EXPONENT.pow(kale) as i128 * pail).fixed_div_floor(
            &env,
            &(block.pow_zeros as i128),
            &actual_block_reward,
        ) + pail;

        token::StellarAssetClient::new(&env, &asset).mint(&miner, &reward);

        remove_pail(&env, miner.clone(), index);

        extend_instance_ttl(&env);
    }
}

fn generate_hash(
    env: &Env,
    miner: &Address,
    index: &u32,
    nonce: &u128,
    entropy: &BytesN<32>,
) -> BytesN<32> {
    let mut hash_b = [0u8; 84];

    let mut miner_b = [0u8; 32];
    let miner_bytes = miner.clone().to_xdr(env);
    miner_bytes
        .slice(miner_bytes.len() - 32..)
        .copy_into_slice(&mut miner_b);

    hash_b[..4].copy_from_slice(&index.to_be_bytes());
    hash_b[4..4 + 16].copy_from_slice(&nonce.to_be_bytes());
    hash_b[20..20 + 32].copy_from_slice(&entropy.to_array());
    hash_b[52..].copy_from_slice(&miner_b);

    env.crypto()
        .keccak256(&Bytes::from_array(env, &hash_b))
        .to_bytes()
}
