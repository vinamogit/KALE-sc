#![cfg(test)]

use std::println;
extern crate std;

use crate::{
    contract::{Block, Errors, Mine, MineContract, MineContractClient, StorageKeys},
    tests::utils::find_nonce_and_hash,
};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Ledger},
    token, Address, BytesN, Env,
};

// TODO write test utils

#[test]
fn test() {
    let mut env: Env = Env::default();

    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    env.mock_all_auths();

    let mine_address: Address = env.register_contract(None, MineContract);
    let mine_client = MineContractClient::new(&env, &mine_address);

    let token_sac = env.register_stellar_asset_contract_v2(mine_address.clone());
    let token_address = token_sac.address();
    let token_admin = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    let admin: Address = Address::generate(&env);

    mine_client.discover(&admin, &token_address);

    let miner_1: Address = Address::generate(&env);
    let miner_2: Address = Address::generate(&env);
    let miner_3: Address = Address::generate(&env);
    let miner_4: Address = Address::generate(&env);

    let amount_1 = 1_0000000;
    let amount_2 = 0_0001000;
    let amount_3 = 0; // 0_2500000;
    let amount_4 = 0_1000000;

    token_admin.mint(&miner_1, &amount_1);
    token_admin.mint(&miner_2, &amount_2);
    token_admin.mint(&miner_3, &amount_3);
    token_admin.mint(&miner_4, &amount_4);

    mine_client.get_pail(&miner_1, &amount_1);
    mine_client.get_pail(&miner_2, &amount_2);
    mine_client.get_pail(&miner_3, &amount_3);
    mine_client.get_pail(&miner_4, &amount_4);

    let mut mine: Option<Mine> = None;
    let mut block: Option<Block> = None;

    env.as_contract(&mine_address, || {
        mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine);
        block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(mine.clone().unwrap().index));
    });

    let mine = mine.unwrap();
    let block = block.unwrap();

    println!("{:?}", mine);
    println!("{:?}", block);
    print!("\n");

    let (nonce_0, hash_0) = find_nonce_and_hash(&env, &mine.index, &block.entropy, &miner_1, 0);

    let (nonce_1, hash_1) = (
        69873866u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("000000c0fb5b3274139a65cc1130a60dbea02bd8862d5509a0627bf8c7af83be")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_2, hash_2) = (
        1987939534u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("0000000ea28174bd83a8e0bec227b82fa4adb1ff3d68e2e8d0f032f5d74d8a0e")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_3, hash_3) = (
        9282405811u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("00000000c02b6d900f1209addb04dc625f72c438ae55e2c43e016e9b696546c3")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_4, hash_4) = (
        220351956843u128,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("0000000008817a9487f2c2117cd748ce07b889fa12f1a659edd532ee5ba1ce57")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );

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

    mine_client.claim(&miner_1, &mine.index);
    mine_client.claim(&miner_2, &mine.index);
    mine_client.claim(&miner_3, &mine.index);
    mine_client.claim(&miner_4, &mine.index);

    println!(
        "miner 1 profit: {:?}",
        token_client.balance(&miner_1) - amount_1
    );
    println!(
        "miner 2 profit: {:?}",
        token_client.balance(&miner_2) - amount_2
    );
    println!(
        "miner 3 profit: {:?}",
        token_client.balance(&miner_3) - amount_3
    );
    println!(
        "miner 4 profit: {:?}",
        token_client.balance(&miner_4) - amount_4
    );
    print!("\n");

    let mut mine: Option<Mine> = None;
    let mut block: Option<Block> = None;

    env.as_contract(&mine_address, || {
        mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine);

        block = env
            .storage()
            .temporary()
            .get::<StorageKeys, Block>(&StorageKeys::Block(mine.clone().unwrap().index));
    });

    let mine = mine.unwrap();
    let block = block.unwrap();

    println!("{:?}", mine);
    println!("{:?}", block);
}

// TEST update get_kale entry with new zero value
// TEST what happens if you update with a lower zero value (should probably error, or just pick the higher one)
