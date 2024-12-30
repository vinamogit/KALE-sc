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

    let (nonce_0, hash_0) = find_nonce_and_hash(&env, &index, &block.entropy, &farmer_1, 0);
    let (nonce1, hash1) = nonce_hash(
        &env,
        57886u64,
        "0000eca19d51c1adf0cfe8c380b65b9cc62d982fabd61938e678fef5bfd50241",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        133620u64,
        "00002c71b5db7e0a3746cc2280e655e37e421ebc27f40b200a40ec83e613c0f6",
    );

    let (nonce3, hash3) = nonce_hash(
        &env,
        11322u64,
        "000011f478679ca8246ea7866a69102f9e63058b8f85d19eb75744944751c09c",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        27031u64,
        "0000d4ca17515167de2bd5244147e04143a37534214a65ff29fbd9505d7ffa14",
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
        74903,
        "00009c2351d5661634d0b801ed5106a3e49e8dfbdbbbbd65197447e52944bdb2",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        36372,
        "00008db578fb813804369b43a44949603854c64e7b763f4350481c40365aff81",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        179975,
        "0000491a6ea56d7bb7d82ea0d7530a509be90778fc1f0d3a94f43e8e7f632adb",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        19267,
        "0000be5379441d55f11065702a4567b27a5f7e0aec932d5e6add8498883dc38e",
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
        4377,
        "000bb3e9335cd8f7786781b1ed1edef266043ecd85284afc6e91eeaadcdc00ec",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        1899,
        "000546e9e332f8d872839ac8447403acf4672a953fc4ec352bf70c9a5286fd35",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        6504,
        "000434b48c237c61a51d0241483d63a3b7336db8a898b0aec7a93858d315c8dc",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        2250,
        "000173a7152b7c55bac92d4afa8127898282f71ba2fb459e70c15688a6c48c01",
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
        18,
        "0af7d5ec13073005e07e316a014a351eb65158b8d544951d3797265a9a21a23a",
    );
    let (nonce2, hash2) = nonce_hash(
        &env,
        49,
        "00497732d42b27dd0e91942807bd555818366fb9adb0258dac5bdfedc1854d75",
    );
    let (nonce3, hash3) = nonce_hash(
        &env,
        1694,
        "000bc23b991c77c4ce6a97f5d0c6fcb5c5e5052e3155aeca33cc8a1bc0da4a79",
    );
    let (nonce4, hash4) = nonce_hash(
        &env,
        34735,
        "000061bea9fc0b101a0e3e1155e816cdf5541b4aa2d08a702e51cff4e6b1e4f6",
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
            .get::<Storage, Block>(&Storage::Block(index.saturating_sub(1)));
    });

    (index, block.unwrap())
}
