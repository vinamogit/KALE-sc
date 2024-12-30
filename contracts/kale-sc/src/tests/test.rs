#![cfg(test)]

use std::println;
extern crate std;

use crate::{
    errors::Errors,
    tests::utils::find_nonce_and_hash,
    types::{Block, Storage},
    Contract, ContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Ledger},
    token, Address, BytesN, Env,
};

// TODO add more tests
// TODO write test utils

#[test]
fn test() {
    let mut env: Env = Env::default();

    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    env.ledger().set_min_temp_entry_ttl(17280);
    env.ledger().set_min_persistent_entry_ttl(2073600);

    env.mock_all_auths();

    let sequence = env.ledger().sequence();

    let homesteader: Address = Address::generate(&env);

    let asset_sac = env.register_stellar_asset_contract_v2(homesteader.clone());
    let asset_address = asset_sac.address();
    let asset_homesteader = token::StellarAssetClient::new(&env, &asset_address);
    let asset_client = token::Client::new(&env, &asset_address);

    let farm_address: Address = env.register(Contract, (&homesteader, &asset_address));
    let farm_client = ContractClient::new(&env, &farm_address);

    asset_homesteader.set_admin(&farm_address);

    let farmer_1: Address = Address::generate(&env);
    let farmer_2: Address = Address::generate(&env);
    let farmer_3: Address = Address::generate(&env);
    let farmer_4: Address = Address::generate(&env);

    let amount_1 = 1000;
    let amount_2 = 100;
    let amount_3 = 10;
    let amount_4 = 1;

    asset_homesteader.mint(&farmer_1, &amount_1);
    asset_homesteader.mint(&farmer_2, &amount_2);
    asset_homesteader.mint(&farmer_3, &amount_3);
    asset_homesteader.mint(&farmer_4, &amount_4);

    farm_client.plant(&farmer_1, &amount_1);
    farm_client.plant(&farmer_2, &amount_2);
    farm_client.plant(&farmer_3, &amount_3);
    farm_client.plant(&farmer_4, &amount_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let mut entropy = block.entropy.clone();
    let (nonce_0, hash_0) = find_nonce_and_hash(&env, &index, &entropy, &farmer_1, 0);

    let (nonce1, hash1) = nonce_hash(
        &env,
        102008,
        "0000ffb16dab2537ae0031f506ce9feeeb539a37b6db6fc973fc269b2e537e19",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        19780,
        "00004b7a81ded3b73c692f17b5e30c728a6dc187ab268144cff85f5a7f4c18e9",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        117417,
        "0000b6c398842114d5b9d75a07104c7b9127defbecddbf9dcc3cd99750ef5c52",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        26235,
        "0000b084d2785f738761a0b66cbdbd1946c9428d8f8b84b398c3ac77308327a3",
    );

    env.ledger().set_sequence_number(sequence + 1);

    farm_client.work(&farmer_1, &hash_0, &nonce_0); // 0 zeros
    farm_client.work(&farmer_4, &hash4, &nonce4); // 9 zeros

    // Should not be able to update for a lower or equal zero count
    let err = farm_client
        .try_work(&farmer_1, &hash_0, &nonce_0)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, Errors::ZeroCountTooLow.into());
    // Should be able to update for a higher zero count
    farm_client.work(&farmer_1, &hash1, &nonce1); // 6 zeros

    farm_client.work(&farmer_2, &hash2, &nonce2); // 7 zeros
    farm_client.work(&farmer_3, &hash3, &nonce3); // 8 zeros

    forward(&env, 60);

    farm_client.plant(&farmer_1, &0);
    farm_client.plant(&farmer_2, &0);
    farm_client.plant(&farmer_3, &0);
    farm_client.plant(&farmer_4, &0);

    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    farm_client.harvest(&farmer_4, &index);

    let balance1_1 = asset_client.balance(&farmer_1);
    let balance2_1 = asset_client.balance(&farmer_2);
    let balance3_1 = asset_client.balance(&farmer_3);
    let balance4_1 = asset_client.balance(&farmer_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let (nonce1, hash1) = nonce_hash(
        &env,
        149438,
        "0000cb3a79fbcc9a12cf3ede8ee7cc2c6e676276297489c6950dfe17067f6379",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        44559,
        "0000e73cf2cc7ff5198de0fd5c66bc6cb52cfe59154a690d86e6dcd1c59dc0ad",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        9804,
        "0000c3b47961cc6231f23ce1c8a0111438cd4aa919c5a6b23a240f5518381d7e",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        26485,
        "0000779ac6f03a256378c4f1edd41e193cd2442b8e4f26e5a9d13212dba3af98",
    );

    farm_client.work(&farmer_1, &hash1, &nonce1);
    farm_client.work(&farmer_2, &hash2, &nonce2);
    farm_client.work(&farmer_3, &hash3, &nonce3);
    farm_client.work(&farmer_4, &hash4, &nonce4);

    forward(&env, 62);

    farm_client.plant(&farmer_1, &0);
    farm_client.plant(&farmer_2, &0);
    farm_client.plant(&farmer_3, &0);
    farm_client.plant(&farmer_4, &0);

    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    farm_client.harvest(&farmer_4, &index);

    let balance1_2 = asset_client.balance(&farmer_1);
    let balance2_2 = asset_client.balance(&farmer_2);
    let balance3_2 = asset_client.balance(&farmer_3);
    let balance4_2 = asset_client.balance(&farmer_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let (nonce1, hash1) = nonce_hash(
        &env,
        20910,
        "0001585819ac5deaa3054e9924132245aaf7742b816f1f900e780a692fc08065",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        14593,
        "0005858bb70544d4da49fd716b81c09646150aa213e42ac3c199fbeff6a76888",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        5366,
        "000fc58fd813b11d3acfde193c9fd2e2fd33479ea664550fd9ff25f5fd89cb33",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        7064,
        "0008bb79f1f2e3f4021f9f23cfca315303f54008a41a13ff247ffa74afa8d5f8",
    );

    forward(&env, 10);
    farm_client.work(&farmer_1, &hash1, &nonce1);
    forward(&env, 10);
    farm_client.work(&farmer_2, &hash2, &nonce2);
    forward(&env, 10);
    farm_client.work(&farmer_3, &hash3, &nonce3);
    forward(&env, 10);
    farm_client.work(&farmer_4, &hash4, &nonce4);
    forward(&env, 10);

    forward(&env, 30);
    farm_client.plant(&farmer_1, &0);
    farm_client.plant(&farmer_2, &0);
    farm_client.plant(&farmer_3, &0);
    farm_client.plant(&farmer_4, &0);

    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    farm_client.harvest(&farmer_4, &index);

    let balance1_3 = asset_client.balance(&farmer_1);
    let balance2_3 = asset_client.balance(&farmer_2);
    let balance3_3 = asset_client.balance(&farmer_3);
    let balance4_3 = asset_client.balance(&farmer_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let (nonce1, hash1) = nonce_hash(
        &env,
        4,
        "0aef5d4bcfe428568f737114fa34dd3b88dadb94a04f0550975e8037fdb05b01",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        461,
        "0044164807f6a9d82055d012d032c6c49fd467ec654e57cfc0c53ff6def96a3c",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        3549,
        "000db7768cbf6abd875f1ee5b03a47504ac01d020b6873de07fa5494f2bc2930",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        183120,
        "000087030f833b120608ba875a1b2d087cf4943bd3e438ebc9346553ca5c8b23",
    );

    farm_client.work(&farmer_1, &hash1, &nonce1);
    farm_client.work(&farmer_2, &hash2, &nonce2);
    farm_client.work(&farmer_3, &hash3, &nonce3);
    farm_client.work(&farmer_4, &hash4, &nonce4);

    forward(&env, 70);

    /*
     * Missing work scenario
     */
    let amount_5 = 10000000;
    //farm_client.plant(&farmer_1, &0);

    farm_client.plant(&farmer_1, &amount_5);
    farm_client.plant(&farmer_2, &amount_5);
    farm_client.plant(&farmer_3, &amount_5);
    farm_client.plant(&farmer_4, &amount_5);

    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    farm_client.harvest(&farmer_4, &index);

    let balance1_4 = asset_client.balance(&farmer_1);
    let balance2_4 = asset_client.balance(&farmer_2);
    let balance3_4 = asset_client.balance(&farmer_3);
    let balance4_4 = asset_client.balance(&farmer_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let entropy = block.entropy.clone();
    let (nonce1, hash1) = find_nonce_and_hash(&env, &index, &entropy, &farmer_1, 2);
    let (nonce2, hash2) = find_nonce_and_hash(&env, &index, &entropy, &farmer_2, 2);
    let (nonce3, hash3) = find_nonce_and_hash(&env, &index, &entropy, &farmer_3, 2);

    farm_client.work(&farmer_1, &hash1, &nonce1);
    farm_client.work(&farmer_2, &hash2, &nonce2);
    farm_client.work(&farmer_3, &hash3, &nonce3);
    // Misses the work session farm_client.work(&farmer_4, &hash4, &nonce4);
    forward(&env, 70);
    farm_client.plant(&farmer_1, &0);
    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    let e = farm_client
        .try_harvest(&farmer_4, &index)
        .unwrap_err()
        .unwrap();
    assert_eq!(e, Errors::WorkMissing.into());

    let balance1_5 = asset_client.balance(&farmer_1);
    let balance2_5 = asset_client.balance(&farmer_2);
    let balance3_5 = asset_client.balance(&farmer_3);
    let balance4_5 = asset_client.balance(&farmer_4);

    let (index, block) = get_block(&env, &farm_address);
    println!("Index: {}", index);
    println!("Block: {:?}", block);

    let reward1_1 = balance1_1 - amount_1;
    let reward2_1 = balance2_1 - amount_2;
    let reward3_1 = balance3_1 - amount_3;
    let reward4_1 = balance4_1 - amount_4;

    let reward1_2 = balance1_2 - balance1_1;
    let reward2_2 = balance2_2 - balance2_1;
    let reward3_2 = balance3_2 - balance3_1;
    let reward4_2 = balance4_2 - balance4_1;

    let reward1_3 = balance1_3 - balance1_2;
    let reward2_3 = balance2_3 - balance2_2;
    let reward3_3 = balance3_3 - balance3_2;
    let reward4_3 = balance4_3 - balance4_2;

    let reward1_4 = balance1_4 - balance1_3 + amount_5;
    let reward2_4 = balance2_4 - balance2_3 + amount_5;
    let reward3_4 = balance3_4 - balance3_3 + amount_5;
    let reward4_4 = balance4_4 - balance4_3 + amount_5;

    let reward1_5 = balance1_5 - balance1_4 - amount_5;
    let reward2_5 = balance2_5 - balance2_4 - amount_5;
    let reward3_5 = balance3_5 - balance3_4 - amount_5;
    let reward4_5 = balance4_5 - balance4_4;

    println!(
        "Farmer1: {} | {} | {} | {} | {} | {}",
        reward1_1, reward1_2, reward1_3, reward1_4, reward1_5, balance1_5
    );
    println!(
        "Farmer2: {} | {} | {} | {} | {} | {}",
        reward2_1, reward2_2, reward2_3, reward2_4, reward2_5, balance2_5
    );
    println!(
        "Farmer3: {} | {} | {} | {} | {} | {}",
        reward3_1, reward3_2, reward3_3, reward3_4, reward3_5, balance3_5
    );
    println!(
        "Farmer4: {} | {} | {} | {} | {} | {}",
        reward4_1, reward4_2, reward4_3, reward4_4, reward4_5, balance4_5
    );
    println!(
        "Total: {} | {} | {} | {} | {}",
        reward1_1 + reward2_1 + reward3_1 + reward4_1,
        reward1_2 + reward2_2 + reward3_2 + reward4_2,
        reward1_3 + reward2_3 + reward3_3 + reward4_3,
        reward1_4 + reward2_4 + reward3_4 + reward4_4,
        reward1_5 + reward2_5 + reward3_5 + reward4_5,
    );

    env.as_contract(&farm_address, || {
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(0));
        println!("{:?}", block);
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(1));
        println!("{:?}", block);
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(2));
        println!("{:?}", block);
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(3));
        println!("{:?}", block);
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(4));
        println!("{:?}", block);
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(5));
        println!("{:?}", block);
    });
}

fn forward(env: &Env, ticks: u32) {
    env.ledger()
        .set_sequence_number(env.ledger().get().sequence_number + ticks);
    let time: u64 = (ticks * 5).into();
    env.ledger()
        .set_timestamp(env.ledger().get().timestamp + time);
}
fn nonce_hash(env: &Env, nonce: u64, hash: &str) -> (u64, BytesN<32>) {
    (
        nonce,
        BytesN::<32>::from_array(env, &hex::decode(hash).unwrap().try_into().unwrap()),
    )
}
fn get_block(env: &Env, farm_address: &Address) -> (u32, Block) {
    let mut index = 0;
    let mut block = None;
    env.as_contract(farm_address, || {
        index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::FarmIndex)
            .unwrap_or(0);
        block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index));
    });

    (index, block.unwrap())
}
