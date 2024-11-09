use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Mine {
    pub index: u32,
    pub admin: Address,
    pub asset: Address,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub entropy: BytesN<32>,
    pub next_entropy: BytesN<32>,
    pub pool: u64,
    pub claimed_pool: u64,
    pub pow_zeros: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Storage {
    Mine,
    Block(u32),
    Pail(Address, u32), // miner, index : (stake_i128, Option<zero_count_u32>)
}
