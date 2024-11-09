use soroban_sdk::{contracttype, Address, BytesN};

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
