use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, xdr::ToXdr,
    Address, Bytes, BytesN, Env,
};

// TODO break up code into different chunks / files
// TODO add more comments
// TODO add more tests
// TODO clean up errors
// TODO switch to garden theme vs mining theme

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    AlreadyDiscovered = 1,
    MineNotFound = 2,
    PailAmountTooLow = 3,
    AlreadyHasPail = 4,
    TheMineWasNuked = 5,
    HashIsInvalid = 6,
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
    pub zeros: u64,
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

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128);

    fn claim(env: Env, miner: Address, index: u64);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn fkin_nuke_it(env: Env);
}

const MINER_EXPONENT: u64 = 8; // Higher value gives more weight to zero_count
const BLOCK_REWARD: u64 = 1_0000000;

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
            zeros: 0,
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
                zeros: 0,
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

        // NOTE: we allow passing zeros but zeros further down the stack will cause issues
        // So either A) we should enforce requiring a > 0 value
        // or B) set the min value to 1 (which will cause the interesting side affect of being able to "free" mint 1 stroop of value)
        env.storage()
            .temporary()
            .set::<StorageKeys, i128>(&pail_key, &amount.max(1));

        block.pool += amount as u64;

        env.storage()
            .temporary()
            .set(&StorageKeys::Block(mine.index), &block);

        if amount > 0 {
            token::Client::new(&env, &mine.token).transfer(&miner, &mine.token, &amount);
        }
    }

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128) {
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

        let pail = env
            .storage()
            .temporary()
            .get::<StorageKeys, i128>(&StorageKeys::Pail(miner.clone(), mine.index))
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

        let kale_key = StorageKeys::Kale(miner.clone(), mine.index);

        match env.storage().temporary().get::<StorageKeys, u32>(&kale_key) {
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

        let full_block_reward = BLOCK_REWARD + block.pool;
        let actual_block_reward = (full_block_reward - block.claimed_pool) as i128;

        env.storage().temporary().remove(&kale_key);
        env.storage().temporary().remove(&pail_key);

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
