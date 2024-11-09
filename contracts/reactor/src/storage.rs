use soroban_sdk::{Address, Env};

use crate::{
    types::{Block, Mine, Storage},
    WEEK_OF_LEDGERS,
};

pub fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();

    env.storage()
        .instance()
        .extend_ttl(max_ttl - WEEK_OF_LEDGERS, max_ttl);
}

pub fn has_mine(env: &Env) -> bool {
    env.storage().instance().has::<Storage>(&Storage::Mine)
}
pub fn get_mine(env: &Env) -> Option<Mine> {
    env.storage()
        .instance()
        .get::<Storage, Mine>(&Storage::Mine)
}
pub fn set_mine(env: &Env, mine: &Mine) {
    env.storage()
        .instance()
        .set::<Storage, Mine>(&Storage::Mine, mine);
}

pub fn get_block(env: &Env, index: u32) -> Option<Block> {
    env.storage()
        .temporary()
        .get::<Storage, Block>(&Storage::Block(index))
}
pub fn set_block(env: &Env, index: u32, block: &Block) {
    env.storage()
        .temporary()
        .set::<Storage, Block>(&Storage::Block(index), block);
}

pub fn has_pail(env: &Env, miner: Address, index: u32) -> bool {
    let pail_key = Storage::Pail(miner, index);

    env.storage().temporary().has::<Storage>(&pail_key)
}
pub fn get_pail(env: &Env, miner: Address, index: u32) -> Option<(i128, Option<u32>)> {
    let pail_key = Storage::Pail(miner, index);

    env.storage()
        .temporary()
        .get::<Storage, (i128, Option<u32>)>(&pail_key)
}
pub fn set_pail(env: &Env, miner: Address, index: u32, amount: i128, zeros: Option<u32>) {
    let pail_key = Storage::Pail(miner, index);

    // NOTE: we allow passing zeros but zeros further down the stack will cause issues
    // So either A) we should enforce requiring a > 0 value
    // or B) set the min value to 1 (which will cause the interesting side affect of being able to "free" mint 1 stroop of value)
    env.storage()
        .temporary()
        .set::<Storage, (i128, Option<u32>)>(&pail_key, &(amount.max(1), zeros));
}
pub fn remove_pail(env: &Env, miner: Address, index: u32) {
    let pail_key = Storage::Pail(miner, index);

    env.storage().temporary().remove::<Storage>(&pail_key);
}
