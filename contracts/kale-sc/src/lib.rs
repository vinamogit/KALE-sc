#![no_std]

use soroban_sdk::{contract, Address, BytesN, Env};

mod contract_farm;
mod contract_homestead;
mod errors;
mod storage;
mod tests;
mod types;

// Higher BLOCK_INTERVAL means there's more time to mine a block which means we can have more participation without risking problematic congestion
// Too low and the network could be congested
// Too high and there's too much time for fast miners to dominate
pub const BLOCK_INTERVAL: u64 = 60 * 5; // In seconds
pub const BLOCK_REWARD: i128 = 1_0000000 * BLOCK_INTERVAL as i128 / 60; // base_per_minute * second_interval / seconds_per_minute
pub const WEEK_OF_LEDGERS: u32 = 60 * 60 * 24 / 5 * 7; // assumes 5 second ledger close times

// TODO add more comments

#[contract]
pub struct Contract;

pub trait HomesteadTrait {
    fn __constructor(env: Env, farmer: Address, asset: Address);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn pause(env: Env);

    fn unpause(env: Env);

    fn remove_block(env: Env, index: u32);
}

pub trait FarmTrait {
    fn plant(env: Env, farmer: Address, amount: i128);

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u64) -> u32;

    fn harvest(env: Env, farmer: Address, index: u32) -> i128;
}
