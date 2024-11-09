#![no_std]

use soroban_sdk::{contract, Address, BytesN, Env};

mod contract_kalepail;
mod contract_mine;
mod errors;
mod storage;
mod tests;
mod types;

pub const MINER_EXPONENT: u64 = 8; // Higher value gives more weight to zero_count
pub const BLOCK_REWARD: u64 = 1_0000000;
pub const WEEK_OF_LEDGERS: u32 = 60 * 60 * 24 / 5 * 7;

// TODO add more comments
// TODO switch to garden theme vs mining theme

#[contract]
pub struct MineKalepailContract;

pub trait MineContractTrait {
    fn discover_mine(env: Env, admin: Address, asset: Address);

    fn pause_mine(env: Env);

    fn unpause_mine(env: Env);

    fn upgrade_mine(env: Env, hash: BytesN<32>);
}

pub trait KalepailTrait {
    fn get_pail(env: Env, miner: Address, amount: i128);

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128);

    fn claim_kale(env: Env, miner: Address, index: u32);
}
