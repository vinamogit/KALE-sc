#![no_std]

use soroban_sdk::{Address, BytesN, Env};

mod contract;
mod errors;
mod storage;
mod tests;
mod types;

pub const MINER_EXPONENT: u64 = 8; // Higher value gives more weight to zero_count
pub const BLOCK_REWARD: u64 = 1_0000000;
pub const WEEK_OF_LEDGERS: u32 = 60 * 60 * 24 / 5 * 7;

// TODO add more comments
// TODO switch to garden theme vs mining theme

pub trait MineContractTrait {
    fn discover(env: Env, admin: Address, token: Address);

    fn get_pail(env: Env, miner: Address, amount: i128);

    fn get_kale(env: Env, miner: Address, hash: BytesN<32>, nonce: u128);

    fn claim(env: Env, miner: Address, index: u64);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn pause(env: Env);

    fn unpause(env: Env);
}
