use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, xdr::ToXdr,
    Address, Bytes, BytesN, Env,
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
    pub zeros: i128,
    pub entropy: BytesN<32>,
    pub total_pool: i128,
    pub claimed_pool: i128,
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
    fn discover(env: Env, miner: Address, token: Address);

    fn get_pail(env: Env, miner: Address, amount: i128);

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: i128);

    fn claim(env: Env, miner: Address, index: u64);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn fkin_nuke_it(env: Env);
}

const ZERO_EXPONENT: i128 = 8;

#[contract]
pub struct MineContract;

#[contractimpl]
impl MineContractTrait for MineContract {
    fn discover(env: Env, miner: Address, token: Address) {
        miner.require_auth();

        if env
            .storage()
            .instance()
            .has::<StorageKeys>(&StorageKeys::Mine)
        {
            panic_with_error!(&env, &Errors::AlreadyDiscovered);
        }

        let mine = Mine {
            index: 0,
            admin: miner,
            token,
            nuked: false,
        };
        let block = Block {
            timestamp: 0,
            zeros: 0,
            entropy: BytesN::from_array(&env, &[0; 32]),
            total_pool: 0,
            claimed_pool: 0,
        };

        env.storage().instance().set(&StorageKeys::Mine, &mine);
        env.storage()
            .temporary()
            .set(&StorageKeys::Block(mine.index), &block);
    }

    fn get_pail(env: Env, miner: Address, amount: i128) {
        miner.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, &Errors::PailAmountTooLow);
        }

        let mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        let mut block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(mine.index))
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let pail_key = StorageKeys::Pail(miner.clone(), mine.index);

        if env.storage().temporary().has::<StorageKeys>(&pail_key) {
            panic_with_error!(&env, &Errors::AlreadyHasPail);
        }

        env.storage()
            .temporary()
            .set::<StorageKeys, i128>(&pail_key, &amount);

        block.total_pool += amount;

        env.storage()
            .temporary()
            .set(&StorageKeys::Block(mine.index), &block);

        token::Client::new(&env, &mine.token).transfer(
            &miner,
            &env.current_contract_address(),
            &amount,
        );
    }

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: i128) {
        miner.require_auth();

        let mut mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.nuked {
            panic_with_error!(&env, &Errors::TheMineWasNuked);
        }

        let mut block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(mine.index))
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockNotFound));

        let mut hash_b = [0u8; 144];

        let mut mine_b = [0u8; 44];
        env.current_contract_address()
            .to_xdr(&env)
            .copy_into_slice(&mut mine_b);

        let mut miner_b = [0u8; 44];
        miner.clone().to_xdr(&env).copy_into_slice(&mut miner_b);

        let index_b = mine.index.to_be_bytes();
        let nonce_b = nonce.to_be_bytes();

        hash_b[0..44].copy_from_slice(&mine_b);
        hash_b[44..44 + 44].copy_from_slice(&miner_b);
        hash_b[88..88 + 8].copy_from_slice(&index_b);
        hash_b[96..96 + 16].copy_from_slice(&nonce_b);
        hash_b[112..112 + 32].copy_from_slice(&block.entropy.to_array());

        let generated_hash = env
            .crypto()
            .keccak256(&Bytes::from_array(&env, &hash_b))
            .to_bytes();

        if hash != generated_hash {
            panic_with_error!(&env, &Errors::ProvidedHashIsInvalid);
        }

        let mut zero_count = 0;

        for byte in hash.iter() {
            if byte == 0 {
                zero_count += 1;
            } else {
                break;
            }
        }

        let kale_key = StorageKeys::Kale(miner.clone(), mine.index);

        match env.storage().temporary().get::<StorageKeys, u32>(&kale_key) {
            Some(prev_zero_count) => {
                block.zeros += ZERO_EXPONENT.pow(zero_count) - ZERO_EXPONENT.pow(prev_zero_count);
            }
            None => {
                let pail = env
                    .storage()
                    .temporary()
                    .get::<StorageKeys, i128>(&StorageKeys::Pail(miner.clone(), mine.index))
                    .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailNotFound));

                block.zeros += ZERO_EXPONENT.pow(zero_count);
                block.claimed_pool -= pail;
            }
        }

        env.storage()
            .temporary()
            .set::<StorageKeys, u32>(&kale_key, &zero_count);

        let next_block_key = StorageKeys::Block(mine.index + 1);

        if env.ledger().timestamp() >= block.timestamp + 60
            && !env.storage().temporary().has(&next_block_key)
        {
            mine.index += 1;

            let next_block = Block {
                timestamp: env.ledger().timestamp(),
                zeros: 0,
                entropy: hash,
                total_pool: 0,
                claimed_pool: 0,
            };

            env.storage().instance().set(&StorageKeys::Mine, &mine);
            env.storage().temporary().set(&next_block_key, &next_block);
        }
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
            .get::<StorageKeys, i128>(&kale_key)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::KaleNotFound));

        let block_reward = 1_0000000 + block.total_pool - block.claimed_pool;

        env.storage().temporary().remove(&kale_key);
        env.storage().temporary().remove(&pail_key);

        let reward = ((kale + pail) / (block.zeros + block.claimed_pool)) * block_reward;

        token::Client::new(&env, &mine.token).transfer(
            &env.current_contract_address(),
            &miner,
            &reward,
        );
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
