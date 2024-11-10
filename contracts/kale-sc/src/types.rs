use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,
    pub entropy: BytesN<32>,
    pub staked: u64,
    pub reclaimed: u64,
    pub pow_zeros: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Storage {
    Homesteader,        // : address
    HomesteadAsset,     // : address
    FarmIndex,          // : u32
    FarmEntropy,        // : bytes32
    FarmPaused,         // : bool
    Block(u32),         // : Block
    Pail(Address, u32), // farmer, index : (stake_i128, Option<zero_count_u32>)
}
