use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub timestamp: u64,

    pub min_gap: u32,
    pub min_stake: i128,
    pub min_zeros: u32,

    pub max_gap: u32,
    pub max_stake: i128,
    pub max_zeros: u32,

    pub entropy: BytesN<32>,

    pub staked_total: i128,
    pub normalized_total: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Pail {
    pub sequence: u32,
    pub gap: Option<u32>,
    pub stake: i128,
    pub zeros: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Storage {
    Homesteader,    // : address
    HomesteadAsset, // : address
    // HomesteadBlockInterval, // : u64
    // HomesteadBlockReward,   // : u64
    FarmIndex,          // : u32
    FarmBlock,          // : Block
    FarmPaused,         // : bool
    Block(u32),         // (index) : Block
    Pail(Address, u32), // (farmer, index) : Pail
}
