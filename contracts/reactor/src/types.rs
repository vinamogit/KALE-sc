use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub entropy: BytesN<32>,
    pub pool: u64,
    pub claimed_pool: u64,
    pub pow_zeros: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Storage {
    MineAdmin,          // : address
    MineAsset,          // : address
    MineIndex,          // : u32
    MineEntropy,        // : bytes32
    MinePaused,         // : bool
    Block(u32),         // : Block
    Pail(Address, u32), // miner, index : (stake_i128, Option<zero_count_u32>)
}
