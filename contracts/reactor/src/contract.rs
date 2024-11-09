use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env,
};

use crate::{
    errors::Errors,
    storage::{
        extend_instance_ttl, get_block, get_kail, get_mine, get_pail, has_mine, has_pail,
        remove_kail, remove_pail, set_block, set_kail, set_mine, set_pail,
    },
    types::{Block, Mine},
    MineContractTrait, BLOCK_REWARD, MINER_EXPONENT,
};

#[contract]
pub struct MineContract;

#[contractimpl]
impl MineContractTrait for MineContract {
    fn discover(env: Env, admin: Address, token: Address) {
        admin.require_auth();

        if has_mine(&env) {
            panic_with_error!(&env, &Errors::AlreadyDiscovered);
        }

        let mine = Mine {
            index: 0,
            admin,
            token,
            paused: false,
        };
        let entropy = BytesN::from_array(&env, &[0; 32]);
        let block = Block {
            timestamp: 0,
            zeros: 0,
            entropy: entropy.clone(),
            next_entropy: entropy,
            pool: 0,
            claimed_pool: 0,
        };

        set_mine(&env, &mine);
        set_block(&env, mine.index, &block);

        extend_instance_ttl(&env);
    }

    fn get_pail(env: Env, miner: Address, amount: i128) {
        miner.require_auth();

        if amount < 0 {
            panic_with_error!(&env, &Errors::PailAmountTooLow);
        }

        let mut mine =
            get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.paused {
            panic_with_error!(&env, &Errors::MineIsPaused);
        }

        let mut block = get_block(&env, mine.index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        if env.ledger().timestamp() >= block.timestamp + 60 {
            mine.index += 1;

            block = Block {
                timestamp: env.ledger().timestamp(),
                zeros: 0,
                entropy: block.next_entropy,
                next_entropy: BytesN::from_array(&env, &[0; 32]),
                pool: 0,
                claimed_pool: 0,
            };

            set_mine(&env, &mine);
        }

        if has_pail(&env, miner.clone(), mine.index) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        // NOTE consider adding a zero_count commitment to the pail vs just a stake amount
        // This would ensure folks couldn't run a lot of initial get_kale's for low zero counts as they tried to find a highest
        // I think initially though I want to try this version and see what happens

        set_pail(&env, miner.clone(), mine.index, amount);

        block.pool += amount as u64;

        set_block(&env, mine.index, &block);

        if amount > 0 {
            token::Client::new(&env, &mine.token).transfer(&miner, &mine.token, &amount);
        }

        extend_instance_ttl(&env);
    }

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128) {
        miner.require_auth();

        let mine = get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        let mut block = get_block(&env, mine.index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let mut hash_b = [0u8; 88];

        let mut miner_b = [0u8; 32];
        let miner_bytes = miner.clone().to_xdr(&env);
        miner_bytes
            .slice(miner_bytes.len() - 32..)
            .copy_into_slice(&mut miner_b);

        hash_b[0..8].copy_from_slice(&mine.index.to_be_bytes());
        hash_b[8..8 + 16].copy_from_slice(&nonce.to_be_bytes());
        hash_b[24..24 + 32].copy_from_slice(&block.entropy.to_array());
        hash_b[56..56 + 32].copy_from_slice(&miner_b);

        let generated_hash = env
            .crypto()
            .keccak256(&Bytes::from_array(&env, &hash_b))
            .to_bytes();

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::HashIsInvalid);
        }

        block.next_entropy = generated_hash;

        let mut zero_count = 0;

        for byte in hash {
            if byte == 0 {
                zero_count += 2;
            } else {
                zero_count += byte.leading_zeros() / 4;
                break;
            }
        }

        let pail = get_pail(&env, miner.clone(), mine.index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        match get_kail(&env, miner.clone(), mine.index) {
            Some(prev_zero_count) => {
                if zero_count <= prev_zero_count {
                    panic_with_error!(&env, &Errors::ZeroCountTooLow);
                }

                block.zeros = block.zeros + (MINER_EXPONENT.pow(zero_count) * pail as u64)
                    - (MINER_EXPONENT.pow(prev_zero_count) * pail as u64);
            }
            None => {
                block.zeros = block.zeros + (MINER_EXPONENT.pow(zero_count) * pail as u64);
                block.claimed_pool += pail as u64;
            }
        }

        set_kail(&env, miner, mine.index, &zero_count);
        set_block(&env, mine.index, &block);

        extend_instance_ttl(&env);
    }

    fn claim(env: Env, miner: Address, index: u64) {
        let mine = get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if index >= mine.index {
            panic_with_error!(&env, &Errors::TooSoonToClaim);
        }

        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let pail = get_pail(&env, miner.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        let kale = get_kail(&env, miner.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::KaleNotFound));

        let full_block_reward = BLOCK_REWARD + block.pool;
        let actual_block_reward = (full_block_reward - block.claimed_pool) as i128;

        remove_kail(&env, miner.clone(), index);
        remove_pail(&env, miner.clone(), index);

        // println!("kale {:?}", kale);
        // println!("pail {:?}", pail);

        // println!("block.zeros {:?}", block.zeros);
        // println!("full_block_reward {:?}", full_block_reward);

        // print!("\n");

        let reward = (MINER_EXPONENT.pow(kale) as i128 * pail).fixed_div_floor(
            &env,
            &(block.zeros as i128),
            &actual_block_reward,
        ) + pail;

        token::StellarAssetClient::new(&env, &mine.token).mint(&miner, &reward);

        extend_instance_ttl(&env);
    }

    fn upgrade(env: Env, hash: BytesN<32>) {
        let mine = get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        mine.admin.require_auth();

        env.deployer().update_current_contract_wasm(hash);

        extend_instance_ttl(&env);
    }

    fn pause(env: Env) {
        let mut mine =
            get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.paused {
            panic_with_error!(&env, &Errors::MineIsPaused);
        }

        mine.admin.require_auth();

        mine.paused = true;

        set_mine(&env, &mine);
    }

    fn unpause(env: Env) {
        let mut mine =
            get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        mine.admin.require_auth();

        mine.paused = false;

        set_mine(&env, &mine);
    }
}
