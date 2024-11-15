#![cfg(test)]

use std::println;
extern crate std;

use ed25519_dalek::Keypair;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};
use stellar_strkey::{ed25519, Strkey};
use tiny_keccak::{Hasher, Keccak};

use crate::Contract;

pub fn find_nonce_and_hash(
    env: &Env,
    index: &u32,
    entropy: &BytesN<32>,
    farmer: &Address,
    zeros: u32,
) -> (u64, BytesN<32>) {
    let mut nonce = 0;
    let mut hash_array = generate_hash(env, index, &nonce, entropy, farmer);

    // println!("{:?}", hash_array);

    loop {
        let hash = generate_keccak(&mut hash_array, &nonce);
        let mut leading_zeros = 0;

        for byte in hash {
            if byte == 0 {
                leading_zeros += 2;
            } else {
                // Use leading_zeros to count bits, convert to hex digits
                leading_zeros += byte.leading_zeros() / 4;
                break;
            }
        }

        if leading_zeros >= zeros {
            return (nonce, BytesN::from_array(env, &hash));
        }

        nonce += 1;
    }
}

fn generate_keccak(hash_b: &mut [u8; 76], nonce: &u64) -> [u8; 32] {
    let mut hash = [0u8; 32];

    hash_b[8..16].copy_from_slice(&nonce.to_be_bytes());

    let mut keccak = Keccak::v256();
    keccak.update(hash_b);
    keccak.finalize(&mut hash);

    hash
}

fn generate_hash(
    env: &Env,
    index: &u32,
    nonce: &u64,
    entropy: &BytesN<32>,
    farmer: &Address,
) -> [u8; 76] {
    let mut hash_array = [0u8; 76];

    let mut farmer_array = [0u8; 32];
    let farmer_bytes = farmer.to_xdr(env);
    farmer_bytes
        .slice(farmer_bytes.len() - 32..)
        .copy_into_slice(&mut farmer_array);

    hash_array[..4].copy_from_slice(&index.to_be_bytes());
    hash_array[4..12].copy_from_slice(&nonce.to_be_bytes());
    hash_array[12..44].copy_from_slice(&entropy.to_array());
    hash_array[44..].copy_from_slice(&farmer_array);

    hash_array
}
fn integer_nth_root(y: u64, n: u32) -> u64 {
    if y == 0 {
        return 0;
    }

    if y == 1 || n == 1 {
        return y;
    }

    let mut low = 1;
    let mut high = y;

    while low < high {
        let mid = (low + high) / 2;

        // Calculate mid^n using integer multiplication
        let mut power = 1u64;
        let mut overflow = false;

        for _ in 0..n {
            match power.checked_mul(mid) {
                Some(val) if val <= y => power = val,
                _ => {
                    overflow = true;
                    break;
                }
            }
        }

        if !overflow && power == y {
            return mid; // Exact match found
        } else if !overflow && power < y {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    low - 1
}

#[test]
fn test() {
    let max_zero: i128 = 9;
    let min_zero: i128 = 5;
    let my_zero: i128 = 4;

    let range_zero = max_zero - min_zero;

    println!(
        "{:?}",
        100_0000000.fixed_div_floor(range_zero, my_zero - min_zero)
    );

    let max_stake: i128 = 100_0000000;
    let min_stake: i128 = 1;
    let my_stake: i128 = 10_0000000;

    let range_stake = max_stake - min_stake;

    println!(
        "{:?}",
        100_0000000.fixed_div_floor(range_stake, my_stake - min_stake)
    );

    let max_gap: i128 = 60;
    let min_gap: i128 = 1;
    let my_gap: i128 = 40;

    let range_gap = max_gap - min_gap;

    println!(
        "{:?}",
        100_0000000.fixed_div_floor(range_gap, my_gap - min_gap)
    );
}

#[test]
fn test_address_lengths() {
    let env: Env = Env::default();

    let farm_address: Address = env.register_contract(None, Contract);

    let ed25519_keypair = Keypair::from_bytes(&[
        149, 154, 40, 132, 13, 234, 167, 87, 182, 44, 152, 45, 242, 179, 187, 17, 139, 106, 49, 85,
        249, 235, 17, 248, 24, 170, 19, 164, 23, 117, 145, 252, 172, 35, 170, 26, 69, 15, 75, 127,
        192, 170, 166, 54, 68, 127, 218, 29, 130, 173, 159, 1, 253, 192, 48, 242, 80, 12, 55, 152,
        223, 122, 198, 96,
    ])
    .unwrap();

    let ed25519_strkey =
        Strkey::PublicKeyEd25519(ed25519::PublicKey(ed25519_keypair.public.to_bytes()));
    let ed25519_address = Bytes::from_slice(&env, ed25519_strkey.to_string().as_bytes());
    let ed25519_address = Address::from_string_bytes(&ed25519_address);

    println!(
        "g-{:?} {:?}",
        ed25519_address.clone().to_xdr(&env).len(),
        ed25519_address.to_string()
    );
    println!(
        "c-{:?} {:?}",
        farm_address.clone().to_xdr(&env).len(),
        farm_address.to_string()
    );
}

#[test]
fn count_bytes() {
    let env = Env::default();

    let bytesn: BytesN<32> = BytesN::from_array(
        &env,
        &hex::decode("00000000075122504cde7b56f7a295ab5588a88eb74f87048d1c0ff9ec083bcb")
            .unwrap()
            .try_into()
            .unwrap(),
    );

    let mut count = 0;

    for byte in bytesn {
        if byte == 0 {
            count += 2;
        } else {
            // Use leading_zeros to count bits, convert to hex digits
            count += byte.leading_zeros() / 4;
            break;
        }
    }

    println!("{:?}", count);
}

#[test]
fn test_fixed_div_floor() {
    let x: i128 = 10;
    let y: i128 = 100;
    let denominator: i128 = 1_0000000;

    let res = x.fixed_div_floor(y, denominator);

    println!("{:?}", res);
}

#[test]
fn test_integer_nth_root() {
    let y = 1_0000000;
    let n = 8;

    let res = integer_nth_root(y, n);

    println!("{:?}", res);
}
