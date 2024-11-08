#![cfg(test)]

use std::println;
extern crate std;

use crate::{
    contract::{Mine, MineContract, MineContractClient, StorageKeys},
    tests::utils::find_nonce_and_hash,
};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger},
    token,
    xdr::ToXdr,
    Address, BytesN, Env,
};

#[test]
fn test() {
    let env: Env = Env::default();

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
    // let miner_4: Address = Address::generate(&env);

    let amount_1 = 10_0000000;
    let amount_2 = 0;
    let amount_3 = 0;

    token_admin.mint(&miner_1, &amount_1);
    token_admin.mint(&miner_2, &amount_2);
    token_admin.mint(&miner_3, &amount_3);
    // token_admin.mint(&miner_4, &amount_4);

    mine_client.get_pail(&miner_1, &amount_1);
    mine_client.get_pail(&miner_2, &amount_2);
    mine_client.get_pail(&miner_3, &amount_3);
    // mine_client.get_pail(&miner_4, &amount_4);

    let mut mine: Option<Mine> = None;

    env.as_contract(&mine_address, || {
        mine = env
            .storage()
            .instance()
            .get::<StorageKeys, Mine>(&StorageKeys::Mine);
    });

    let mine = mine.unwrap();

    // let entropy = BytesN::from_array(&env, &[0u8; 32]);

    // env.budget().reset_unlimited();

    let (nonce_1, hash_1) = (
        36893488190368804358,
        BytesN::from_array(
            &env,
            &hex::decode("000000896593bf9bf63f8614059d62bfb676d63feed5450baf5b9aec418993e8")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_2, hash_2) = (
        73786976823119184377,
        BytesN::from_array(
            &env,
            &hex::decode("0000000fe60e3da185e521cb13a4549b9eb4cf5c117e745069870f79c532284e")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_3, hash_3) = (
        12873150361,
        BytesN::from_array(
            &env,
            &hex::decode("00000000db185e6599d7e3f39516c6432ada178e7576cba6930abd653d968f3a")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    // let (nonce_4, hash_4) = (4500131231, BytesN::from_array(&env, &hex::decode("00000000075122504cde7b56f7a295ab5588a88eb74f87048d1c0ff9ec083bcb").unwrap().try_into().unwrap()));

    // env.budget().reset_unlimited();

    mine_client.get_kale(&miner_1, &hash_1, &nonce_1);
    mine_client.get_kale(&miner_2, &hash_2, &nonce_2);
    mine_client.get_kale(&miner_3, &hash_3, &nonce_3);
    // mine_client.get_kale(&miner_4, &hash_4, &nonce_4);

    env.ledger().set_timestamp(env.ledger().timestamp() + 60);

    mine_client.get_pail(&miner_1, &0);

    mine_client.claim(&miner_1, &mine.index);
    mine_client.claim(&miner_2, &mine.index);
    mine_client.claim(&miner_3, &mine.index);
    // mine_client.claim(&miner_4, &mine.index);

    println!("miner 1: {:?}", token_client.balance(&miner_1));
    println!("miner 2: {:?}", token_client.balance(&miner_2));
    println!("miner 3: {:?}", token_client.balance(&miner_3));
    // println!("miner 4: {:?}", token_client.balance(&miner_4));
}

// ensure if you get_pail and get_kale you never end up being able to claim for less than your get_pail stake
// update get_kale entry with new zero value
// what happens if you update with a lower zero value (should probably error, or just pick the higher one)

// get_kale without get_pail

// claim without get_pail
// claim without get_kale

// claim too soon
// dupe claim

// TEST assuming G-address and C-address are both 40 bytes long when converting from XDR
