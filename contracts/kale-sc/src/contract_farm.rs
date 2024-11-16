// use crate::ContractArgs;
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        bump_farm_index, extend_instance_ttl, get_block, get_farm_asset, get_farm_block,
        get_farm_index, get_farm_paused, get_pail, has_pail, remove_pail, set_block,
        set_farm_block, set_pail,
    },
    types::{Block, Pail},
    Contract, ContractClient, FarmTrait, BLOCK_INTERVAL, BLOCK_REWARD,
};

#[contractimpl]
impl FarmTrait for Contract {
    fn plant(env: Env, farmer: Address, amount: i128) {
        farmer.require_auth();

        let asset = get_farm_asset(&env);
        let mut index = get_farm_index(&env);
        let mut farm_block = get_farm_block(&env).unwrap_or(empty_block(&env));
        let paused = get_farm_paused(&env);
        let mut block = match get_block(&env, index) {
            // genesis or evicted
            None => {
                // we're in an evicted scenario so the index should be bumped
                if index > 0 {
                    bump_farm_index(&env, &mut index);
                }

                // set with some reasonable defaults
                Block {
                    timestamp: env.ledger().timestamp(),
                    min_gap: 1,
                    min_stake: 0,
                    min_zeros: 7,
                    max_gap: 60,
                    max_stake: 5_0000000,
                    max_zeros: 10,
                    entropy: BytesN::from_array(&env, &[0; 32]),
                    staked_total: 0,
                    normalized_total: 0,
                }
            }
            Some(block) => {
                // if the block is >= BLOCK_INTERVAL old, we need to create a new one
                if env.ledger().timestamp() >= block.timestamp + BLOCK_INTERVAL {
                    // initialize with the values from the previous block
                    let mut block = farm_block.clone();

                    block.timestamp = env.ledger().timestamp();
                    block.staked_total = 0;
                    block.normalized_total = 0;

                    // ensure we put this after the `farm_block.clone` above
                    bump_farm_index(&env, &mut index);
                    farm_block = empty_block(&env);

                    block
                } else {
                    block
                }
            }
        };

        if paused {
            panic_with_error!(&env, &Errors::FarmPaused);
        }

        if amount < 0 {
            panic_with_error!(&env, &Errors::PlantAmountTooLow);
        }

        // must come after block discovery as the index may have been bumped
        if has_pail(&env, farmer.clone(), index) {
            panic_with_error!(&env, &Errors::PailExists);
        }

        block.staked_total += amount;

        if amount > 0 {
            token::Client::new(&env, &asset).burn(&farmer, &amount);
        }

        if amount > farm_block.max_stake {
            farm_block.max_stake = amount;
        }

        if amount < farm_block.min_stake {
            farm_block.min_stake = amount;
        }

        let pail = Pail {
            sequence: env.ledger().sequence(),
            gap: None,
            stake: amount,
            zeros: None,
        };

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);
        set_farm_block(&env, &farm_block);

        extend_instance_ttl(&env);
    }

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u64) -> u32 {
        // No auth_require here so others can call this function on the `farmer`'s behalf

        let index = get_farm_index(&env);
        let mut farm_block = get_farm_block(&env)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::HomesteadMissing));
        let mut block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockMissing));
        let mut pail = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailMissing));

        let generated_hash = generate_hash(&env, &index, &nonce, &block.entropy, &farmer);
        let sequence = env.ledger().sequence();
        let gap = sequence - pail.sequence;
        let mut zeros = 0;

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::HashInvalid);
        }

        for byte in hash {
            if byte == 0 {
                zeros += 2;
            } else {
                zeros += byte.leading_zeros() / 4;
                break;
            }
        }

        let (normalized_gap, normalized_stake, normalized_zero) =
            generate_normalizations(&env, &block, gap, pail.stake, zeros);

        block.normalized_total += normalized_gap + normalized_stake + normalized_zero;

        match pail.zeros {
            Some(prev_zeros) => {
                if zeros <= prev_zeros {
                    panic_with_error!(&env, &Errors::ZeroCountTooLow);
                }

                let (prev_normalized_gap, prev_normalized_stake, prev_normalized_zero) =
                    generate_normalizations(&env, &block, gap, pail.stake, prev_zeros);

                block.normalized_total -=
                    prev_normalized_gap + prev_normalized_stake + prev_normalized_zero;
            }
            None => {
                block.staked_total -= pail.stake;
            }
        }

        farm_block.entropy = generated_hash;

        if gap > farm_block.max_gap {
            farm_block.max_gap = gap;
        }

        if gap < farm_block.min_gap {
            farm_block.min_gap = gap;
        }

        if zeros > farm_block.max_zeros {
            farm_block.max_zeros = zeros;
        }

        if zeros < farm_block.min_zeros {
            farm_block.min_zeros = zeros;
        }

        pail.gap = Some(gap);
        pail.zeros = Some(zeros);

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);
        set_farm_block(&env, &farm_block);

        extend_instance_ttl(&env);

        gap
    }

    fn harvest(env: Env, farmer: Address, index: u32) -> i128 {
        let asset = get_farm_asset(&env);
        let farm_index = get_farm_index(&env);
        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockMissing));
        let Pail {
            gap, stake, zeros, ..
        } = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailMissing));

        if index >= farm_index {
            panic_with_error!(&env, &Errors::HarvestNotReady);
        }

        if gap.is_none() || zeros.is_none() {
            panic_with_error!(&env, &Errors::WorkMissing);
        }

        let gap = gap.unwrap();
        let zeros = zeros.unwrap();

        let (normalized_gap, normalized_stake, normalized_zero) =
            generate_normalizations(&env, &block, gap, stake, zeros);

        let reward = (normalized_gap + normalized_stake + normalized_zero).fixed_div_floor(
            &env,
            &block.normalized_total,
            &(BLOCK_REWARD + block.staked_total),
        ) + stake;

        token::StellarAssetClient::new(&env, &asset).mint(&farmer, &reward);

        remove_pail(&env, farmer.clone(), index);

        extend_instance_ttl(&env);

        reward
    }
}

pub fn empty_block(env: &Env) -> Block {
    Block {
        timestamp: env.ledger().timestamp(),
        min_gap: u32::MAX,
        min_stake: i128::MAX,
        min_zeros: u32::MAX,
        max_gap: u32::MIN,
        max_stake: i128::MIN,
        max_zeros: u32::MIN,
        entropy: BytesN::from_array(env, &[0; 32]),
        staked_total: 0,
        normalized_total: 0,
    }
}

fn generate_hash(
    env: &Env,
    index: &u32,
    nonce: &u64,
    entropy: &BytesN<32>,
    farmer: &Address,
) -> BytesN<32> {
    let mut hash_array = [0u8; 76];

    let mut farmer_array = [0u8; 32];
    let farmer_bytes = farmer.to_xdr(env);
    farmer_bytes
        .slice(farmer_bytes.len() - 32..)
        .copy_into_slice(&mut farmer_array);

    hash_array[..4].copy_from_slice(&index.to_be_bytes());
    hash_array[4..12].copy_from_slice(&nonce.to_be_bytes());
    hash_array[12..44].copy_from_slice(&entropy.to_array());
    hash_array[44..].copy_from_slice(&farmer_array);

    env.crypto()
        .keccak256(&Bytes::from_array(env, &hash_array))
        .to_bytes()
}

fn generate_normalizations(
    env: &Env,
    block: &Block,
    gap: u32,
    stake: i128,
    zeros: u32,
) -> (i128, i128, i128) {
    if block.max_gap < block.min_gap
        || block.max_stake < block.min_stake
        || block.max_zeros < block.min_zeros
    {
        panic_with_error!(&env, &Errors::BlockInvalid);
    }

    let range_gap = (block.max_gap - block.min_gap).max(1) as i128;
    let range_stake = block.max_stake - block.min_stake.max(1);
    let range_zero = (block.max_zeros - block.min_zeros).max(1) as i128;

    let ceiling = (block.max_gap as i128)
        .max(block.max_stake)
        .max(block.max_zeros as i128)
        .max(1);

    let gap = gap.min(block.max_gap).max(block.min_gap);
    let stake = stake.min(block.max_stake).max(block.min_stake);
    let zeros = zeros.min(block.max_zeros).max(block.min_zeros);

    let normalized_gap = ceiling.fixed_div_floor(env, &range_gap, &((gap - block.min_gap) as i128));
    let normalized_stake = ceiling.fixed_div_floor(env, &range_stake, &(stake - block.min_stake));
    let normalized_zero =
        ceiling.fixed_div_floor(env, &range_zero, &((zeros - block.min_zeros) as i128));

    (normalized_gap, normalized_stake, normalized_zero)
}
