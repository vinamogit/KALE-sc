use soroban_sdk::{panic_with_error, Address, BytesN, Env};

use crate::{
    errors::Errors,
    types::{Block, Storage},
    WEEK_OF_LEDGERS,
};

pub fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();

    env.storage()
        .instance()
        .extend_ttl(max_ttl - WEEK_OF_LEDGERS, max_ttl);
}

pub fn has_mine_admin(env: &Env) -> bool {
    env.storage().instance().has::<Storage>(&Storage::MineAdmin)
}
pub fn get_mine_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<Storage, Address>(&Storage::MineAdmin)
        .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound))
}
pub fn set_mine_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set::<Storage, Address>(&Storage::MineAdmin, admin);
}

pub fn get_mine_asset(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<Storage, Address>(&Storage::MineAsset)
        .unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound))
}
pub fn set_mine_asset(env: &Env, asset: &Address) {
    env.storage()
        .instance()
        .set::<Storage, Address>(&Storage::MineAsset, asset);
}

pub fn get_mine_index(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get::<Storage, u32>(&Storage::MineIndex)
        .unwrap_or(0)
}
pub fn bump_mine_index(env: &Env, mut current_mine_index: u32) -> u32 {
    current_mine_index += 1;

    env.storage()
        .instance()
        .set::<Storage, u32>(&Storage::MineIndex, &current_mine_index);

    current_mine_index
}

pub fn get_mine_entropy(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get::<Storage, BytesN<32>>(&Storage::MineEntropy)
        .unwrap_or(BytesN::from_array(&env, &[0; 32]))
}
pub fn set_mine_entropy(env: &Env, entropy: &BytesN<32>) {
    env.storage()
        .instance()
        .set::<Storage, BytesN<32>>(&Storage::MineEntropy, entropy);
}

pub fn get_mine_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<Storage, bool>(&Storage::MinePaused)
        .unwrap_or(false)
}
pub fn set_mine_paused(env: &Env, paused: bool) {
    env.storage()
        .instance()
        .set::<Storage, bool>(&Storage::MinePaused, &paused);
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
