use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, xdr::ToXdr,
    Address, Bytes, BytesN, Env, Map,
};

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    AlreadyDiscovered = 1,
    MineNotFound = 2,
    PailAmountTooLow = 3,
    AlreadyHasPail = 4,
    TheMineWasNuked = 5,
    ProvidedHashIsInvalid = 6,
    BlockNotFound = 7,
    TooSoonToClaim = 8,
    KaleNotFound = 9,
    PailNotFound = 10,
    ZeroCountTooLow = 11,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Mine {
    pub index: u64,
    pub admin: Address,
    pub token: Address,
    pub nuked: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub zeros: Map<u32, u64>,
    pub entropy: BytesN<32>,
    pub next_entropy: BytesN<32>,
    pub pool: u64,
    pub claimed_pool: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum StorageKeys {
    Mine,
    Block(u64),
    Pail(Address, u64), // miner, index : (stake_i128)
    Kale(Address, u64), // miner, index : (zero_count_u32)
}

pub trait MineContractTrait {
    fn discover(env: Env, admin: Address, token: Address);

    fn get_pail(env: Env, miner: Address, amount: i128);

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: i128);

    fn claim(env: Env, miner: Address, index: u64);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn fkin_nuke_it(env: Env);
}

#[contract]
pub struct MineContract;

#[contractimpl]
impl MineContractTrait for MineContract {
    fn discover(env: Env, admin: Address, token: Address) {
        admin.require_auth();

        if env
            .storage()
            .instance()
            .has::<StorageKeys>(&StorageKeys::Mine)
        {
            panic_with_error!(&env, &Errors::AlreadyDiscovered);
        }

        let mine = Mine {
            index: 0,
            admin,
            token,
            nuked: false,
        };
        let entropy = BytesN::from_array(&env, &[0; 32]);
        let block = Block {
            timestamp: 0,
            zeros: Map::new(&env),
            entropy: entropy.clone(),
            next_entropy: entropy,
            pool: 0,
            claimed_pool: 0,
        };

        env.storage().instance().set(&StorageKeys::Mine, &mine);
        env.storage()
            .temporary()
            .set(&StorageKeys::Block(mine.index), &block);
    }

    fn get_pail(env: Env, miner: Address, amount: i128) {
        miner.require_auth();

        if amount < 0 {
            panic_with_error!(&env, &Errors::PailAmountTooLow);
        }

        let mut mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        let mut block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(mine.index))
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        if env.ledger().timestamp() >= block.timestamp + 60 {
            mine.index += 1;

            block = Block {
                timestamp: env.ledger().timestamp(),
                zeros: Map::new(&env),
                entropy: block.next_entropy,
                next_entropy: BytesN::from_array(&env, &[0; 32]),
                pool: 0,
                claimed_pool: 0,
            };

            env.storage().instance().set(&StorageKeys::Mine, &mine);
        }

        let pail_key = StorageKeys::Pail(miner.clone(), mine.index);

        if env.storage().temporary().has::<StorageKeys>(&pail_key) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        env.storage()
            .temporary()
            .set::<StorageKeys, i128>(&pail_key, &amount);

        block.pool += amount as u64;

        env.storage()
            .temporary()
            .set(&StorageKeys::Block(mine.index), &block);

        token::Client::new(&env, &mine.token).transfer(&miner, &mine.token, &amount);
    }

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: i128) {
        miner.require_auth();

        let mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.nuked {
            panic_with_error!(&env, &Errors::TheMineWasNuked);
        }

        let block_key = StorageKeys::Block(mine.index);
        let mut block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&block_key)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let mut hash_b = [0u8; 136];

        let mut mine_b = [0u8; 40];
        env.current_contract_address()
            .to_xdr(&env)
            .copy_into_slice(&mut mine_b);

        let mut miner_b = [0u8; 40];
        miner.clone().to_xdr(&env).copy_into_slice(&mut miner_b);

        let index_b = mine.index.to_be_bytes();
        let nonce_b = nonce.to_be_bytes();

        hash_b[0..40].copy_from_slice(&mine_b);
        hash_b[40..40 + 40].copy_from_slice(&miner_b);
        hash_b[80..80 + 8].copy_from_slice(&index_b);
        hash_b[88..88 + 16].copy_from_slice(&nonce_b);
        hash_b[104..104 + 32].copy_from_slice(&block.entropy.to_array());

        let generated_hash = env
            .crypto()
            .keccak256(&Bytes::from_array(&env, &hash_b))
            .to_bytes();

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::ProvidedHashIsInvalid);
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

        if zero_count <= 0 {
            panic_with_error!(&env, &Errors::ZeroCountTooLow);
        }

        let kale_key = StorageKeys::Kale(miner.clone(), mine.index);

        match env.storage().temporary().get::<StorageKeys, u32>(&kale_key) {
            Some(prev_zero_count) => {
                block.zeros.set(
                    prev_zero_count,
                    block.zeros.get_unchecked(prev_zero_count) - 1,
                );
                block
                    .zeros
                    .set(zero_count, block.zeros.get(zero_count).unwrap_or(0) + 1);
            }
            None => {
                let pail = env
                    .storage()
                    .temporary()
                    .get::<StorageKeys, i128>(&StorageKeys::Pail(miner.clone(), mine.index))
                    .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

                block
                    .zeros
                    .set(zero_count, block.zeros.get(zero_count).unwrap_or(0) + 1);
                block.claimed_pool += pail as u64;
            }
        }

        env.storage()
            .temporary()
            .set::<StorageKeys, u32>(&kale_key, &zero_count);
        env.storage().temporary().set(&block_key, &block);
    }

    fn claim(env: Env, miner: Address, index: u64) {
        let mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if index >= mine.index {
            panic_with_error!(&env, &Errors::TooSoonToClaim);
        }

        let block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(index))
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let pail_key = StorageKeys::Pail(miner.clone(), index);
        let pail = env
            .storage()
            .temporary()
            .get::<StorageKeys, i128>(&pail_key)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        let kale_key = StorageKeys::Kale(miner.clone(), index);
        let kale = env
            .storage()
            .temporary()
            .get::<StorageKeys, u32>(&kale_key)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::KaleNotFound));

        let block_reward = 1_0000000;
        let full_block_reward = block_reward + block.pool;
        let actual_block_reward = (full_block_reward - block.claimed_pool) as i128;

        env.storage().temporary().remove(&kale_key);
        env.storage().temporary().remove(&pail_key);

        let largest_zero = block.zeros.keys().last_unchecked();
        let zero_exponent = integer_nth_root(block.pool, largest_zero);

        let mut zeros: u64 = 0;

        println!("largest_zero {:?}", largest_zero);
        println!("zero_exponent {:?}", zero_exponent);

        for (zero_count, miner_count) in block.zeros {
            zeros += zero_exponent.pow(zero_count) * miner_count;
        }

        println!("kale {:?}", zero_exponent.pow(kale));
        println!("pail {:?}", pail);

        println!("zeros {:?}", zeros);
        println!("block.claimed_pool {:?}", block.claimed_pool);

        print!("\n");

        let reward = (zero_exponent.pow(kale) as i128 + pail).fixed_div_floor(
            &env,
            &((zeros + block.claimed_pool).max(1) as i128),
            &actual_block_reward,
        );

        token::StellarAssetClient::new(&env, &mine.token).mint(&miner, &reward);
    }

    fn upgrade(env: Env, hash: BytesN<32>) {
        let mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        mine.admin.require_auth();

        env.deployer().update_current_contract_wasm(hash);
    }

    fn fkin_nuke_it(env: Env) {
        let mut mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.nuked {
            panic_with_error!(&env, &Errors::TheMineWasNuked);
        }

        mine.admin.require_auth();

        mine.nuked = true;

        env.storage().instance().set(&StorageKeys::Mine, &mine);
    }
}

fn integer_nth_root(y: u64, n: u32) -> u64 {
    if y == 0 {
        return 2;
    }

    if y == 1 || n == 1 {
        return y;
    }

    let mut low = 1;
    let mut high = y;

    while low < high {
        let mid = (low + high) / 2;

        // Calculate mid^n using integer multiplication
        let mut power = 1u64;
        let mut overflow = false;

        for _ in 0..n {
            match power.checked_mul(mid) {
                Some(val) if val <= y => power = val,
                _ => {
                    overflow = true;
                    break;
                }
            }
        }

        if !overflow && power == y {
            return mid; // Exact match found
        } else if !overflow && power < y {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    low.max(2)
}
