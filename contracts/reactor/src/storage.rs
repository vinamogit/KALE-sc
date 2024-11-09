use soroban_sdk::{Address, Env};

use crate::{
    types::{Block, Mine, StorageKeys},
    WEEK_OF_LEDGERS,
};

pub fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();

    env.storage()
        .instance()
        .extend_ttl(max_ttl - WEEK_OF_LEDGERS, max_ttl);
}

pub fn has_mine(env: &Env) -> bool {
    env.storage()
        .instance()
        .has::<StorageKeys>(&StorageKeys::Mine)
}
pub fn get_mine(env: &Env) -> Option<Mine> {
    env.storage()
        .instance()
        .get::<StorageKeys, Mine>(&StorageKeys::Mine)
}
pub fn set_mine(env: &Env, mine: &Mine) {
    env.storage()
        .instance()
        .set::<StorageKeys, Mine>(&StorageKeys::Mine, mine);
}

pub fn get_block(env: &Env, index: u64) -> Option<Block> {
    env.storage()
        .temporary()
        .get::<StorageKeys, Block>(&StorageKeys::Block(index))
}
pub fn set_block(env: &Env, index: u64, block: &Block) {
    env.storage()
        .temporary()
        .set::<StorageKeys, Block>(&StorageKeys::Block(index), block);
}

pub fn has_pail(env: &Env, miner: Address, index: u64) -> bool {
    let pail_key = StorageKeys::Pail(miner, index);

    env.storage().temporary().has::<StorageKeys>(&pail_key)
}
pub fn get_pail(env: &Env, miner: Address, index: u64) -> Option<i128> {
    let pail_key = StorageKeys::Pail(miner, index);

    env.storage()
        .temporary()
        .get::<StorageKeys, i128>(&pail_key)
}
pub fn set_pail(env: &Env, miner: Address, index: u64, amount: i128) {
    let pail_key = StorageKeys::Pail(miner, index);

    // NOTE: we allow passing zeros but zeros further down the stack will cause issues
    // So either A) we should enforce requiring a > 0 value
    // or B) set the min value to 1 (which will cause the interesting side affect of being able to "free" mint 1 stroop of value)
    env.storage()
        .temporary()
        .set::<StorageKeys, i128>(&pail_key, &amount.max(1));
}
pub fn remove_pail(env: &Env, miner: Address, index: u64) {
    let pail_key = StorageKeys::Pail(miner, index);

    env.storage().temporary().remove(&pail_key);
}

pub fn get_kail(env: &Env, miner: Address, index: u64) -> Option<u32> {
    let kale_key = StorageKeys::Kale(miner, index);

    env.storage().temporary().get::<StorageKeys, u32>(&kale_key)
}
pub fn set_kail(env: &Env, miner: Address, index: u64, zero_count: &u32) {
    let kale_key = StorageKeys::Kale(miner, index);

    env.storage()
        .temporary()
        .set::<StorageKeys, u32>(&kale_key, zero_count);
}
pub fn remove_kail(env: &Env, miner: Address, index: u64) {
    let kale_key = StorageKeys::Kale(miner, index);

    env.storage().temporary().remove(&kale_key);
}
