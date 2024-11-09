#![no_std]

use soroban_sdk::{contract, Address, BytesN, Env};

mod contract_farm;
mod contract_homestead;
mod errors;
mod storage;
mod tests;
mod types;

pub const ZEROS_EXPONENT: u64 = 8; // Higher value gives more weight to zero_count
pub const BLOCK_REWARD: u64 = 1_0000000;
pub const WEEK_OF_LEDGERS: u32 = 60 * 60 * 24 / 5 * 7;

// TODO add more comments

#[contract]
pub struct Contract;

pub trait HomesteadTrait {
    fn homestead(env: Env, homesteader: Address, asset: Address);

    fn pause(env: Env);

    fn unpause(env: Env);

    fn upgrade(env: Env, hash: BytesN<32>);
}

pub trait FarmTrait {
    fn plant(env: Env, farmer: Address, amount: i128);

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u128);

    fn harvest(env: Env, farmer: Address, index: u32);
}
