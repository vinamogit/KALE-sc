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

// NOTE consider adding a zeros commitment to the Pail vs just a stake amount
// This would ensure folks couldn't run a lot of initial `work`'s for low zero counts as they tried to find a highest
// I think initially though I want to try this version and see what happens
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Pail {
    pub plant_seq: u32,
    pub work_seq: Option<u32>,
    pub stake: i128,
    pub zeros: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Storage {
    Homesteader,        // : address
    HomesteadAsset,     // : address
    FarmIndex,          // : u32
    FarmEntropy,        // : bytes32
    FarmPaused,         // : bool
    Block(u32),         // (index) : Block
    Pail(Address, u32), // (farmer, index) : Pail
}
