#![cfg(test)]

use std::{print, println};
extern crate std;

use crate::{
    errors::Errors,
    tests::utils::find_nonce_and_hash,
    types::{Block, Storage},
    MineKalepailContract, MineKalepailContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Ledger},
    token,
    xdr::ToXdr,
    Address, BytesN, Env,
};

// TODO add more tests
// TODO write test utils

#[test]
fn test() {
    let mut env: Env = Env::default();

    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    env.mock_all_auths();

    let mine_address: Address = env.register_contract(None, MineKalepailContract);
    let mine_client = MineKalepailContractClient::new(&env, &mine_address);

    let asset_sac = env.register_stellar_asset_contract_v2(mine_address.clone());
    let asset_address = asset_sac.address();
    let asset_admin = token::StellarAssetClient::new(&env, &asset_address);
    let asset_client = token::Client::new(&env, &asset_address);

    let admin: Address = Address::generate(&env);

    mine_client.discover_mine(&admin, &asset_address);

    let miner_1: Address = Address::generate(&env);
    let miner_2: Address = Address::generate(&env);
    let miner_3: Address = Address::generate(&env);
    let miner_4: Address = Address::generate(&env);

    let amount_1 = 1_0000000;
    let amount_2 = 0_0001000;
    let amount_3 = 0;
    let amount_4 = 0_1000000;

    asset_admin.mint(&miner_1, &amount_1);
    asset_admin.mint(&miner_2, &amount_2);
    asset_admin.mint(&miner_3, &amount_3);
    asset_admin.mint(&miner_4, &amount_4);

    mine_client.get_pail(&miner_1, &amount_1);
    mine_client.get_pail(&miner_2, &amount_2);
    mine_client.get_pail(&miner_3, &amount_3);
    mine_client.get_pail(&miner_4, &amount_4);

    let mut index: Option<u32> = None;
    let mut block: Option<Block> = None;

    env.as_contract(&mine_address, || {
        index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::MineIndex);
        block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index.unwrap_or(0)));
    });

    let index = index.unwrap_or(0);
    let block = block.unwrap();

    println!("{:?}", index);
    println!("{:?}", block);
    print!("\n");

    let (nonce_0, hash_0) = find_nonce_and_hash(&env, &index, &block.entropy, &miner_1, 0);

    let (nonce_1, hash_1) = (
        101569923u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("000000c49e20bcfd1499b7710243e161a4a55c046fdd81b5590f412a4c72ba7a")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_2, hash_2) = (
        146422264u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("0000000a048c6a47e70d4d470e39a340f6f34c4113dc32fa0595465304b23f29")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_3, hash_3) = (
        1603654064u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("00000000f9997ad594257fe86a5410ab36e96f4d2a04eed577b9fe8aba6f5193")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_4, hash_4) = (
        23177611072u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("000000000f29081bcb654599fd9cc083ca662cf1b5c421433909a7c0abc985e3")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );

    println!("{:?}", miner_1.clone().to_xdr(&env));

    mine_client.get_kale(&miner_1, &hash_0, &nonce_0); // 0 zeros
    mine_client.get_kale(&miner_2, &hash_2, &nonce_2); // 7 zeros
    mine_client.get_kale(&miner_3, &hash_3, &nonce_3); // 8 zeros
    mine_client.get_kale(&miner_4, &hash_4, &nonce_4); // 9 zeros

    // Should be able to update for a higher zero count
    mine_client.get_kale(&miner_1, &hash_1, &nonce_1); // 6 zeros

    // Should not be able to update for a lower zero count
    let err = mine_client
        .try_get_kale(&miner_1, &hash_0, &nonce_0)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, Errors::ZeroCountTooLow.into());

    env.ledger().set_timestamp(env.ledger().timestamp() + 60);

    mine_client.get_pail(&miner_1, &0);

    mine_client.claim_kale(&miner_1, &index);
    mine_client.claim_kale(&miner_2, &index);
    mine_client.claim_kale(&miner_3, &index);
    mine_client.claim_kale(&miner_4, &index);

    println!(
        "miner 1 profit: {:?}",
        asset_client.balance(&miner_1) - amount_1
    );
    println!(
        "miner 2 profit: {:?}",
        asset_client.balance(&miner_2) - amount_2
    );
    println!(
        "miner 3 profit: {:?}",
        asset_client.balance(&miner_3) - amount_3
    );
    println!(
        "miner 4 profit: {:?}",
        asset_client.balance(&miner_4) - amount_4
    );
    print!("\n");

    let mut index: Option<u32> = None;
    let mut block: Option<Block> = None;

    env.as_contract(&mine_address, || {
        index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::MineIndex);

        block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index.unwrap()));
    });

    let index = index.unwrap();
    let block = block.unwrap();

    println!("{:?}", index);
    println!("{:?}", block);
}
