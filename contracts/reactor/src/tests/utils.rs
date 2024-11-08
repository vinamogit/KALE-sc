use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{testutils::BytesN as _, xdr::ToXdr, Address, BytesN, Env};
use tiny_keccak::{Hasher, Keccak};

pub fn find_nonce_and_hash(
    env: &Env,
    mine: &Address,
    miner: &Address,
    index: &u64,
    entropy: &BytesN<32>,
    zero_count: u32,
)
// -> (i128, BytesN<32>)
{
    let mut nonce = 0;
    let mut hash_b = generate_hash(env, mine, miner, index, &nonce, entropy);

    println!("{:?}", hash_b);

    // loop {
    //     let hash = generate_keccak(&mut hash_b, &nonce);
    //     let mut leading_zeros = 0;

    //     for byte in hash.clone() {
    //         if byte == 0 {
    //             leading_zeros += 1;
    //         } else {
    //             break;
    //         }
    //     }

    //     if leading_zeros >= zero_count {
    //         return (nonce, BytesN::from_array(env, &hash));
    //     }

    //     nonce += 1;
    // }
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
    let x: i128 = 64 + 0;
    let y: i128 = 192 + 0;
    let denominator: i128 = 10000000;

    // Math.pow(1_0000000, 1 / 9);

    let res = x.fixed_div_floor(y, denominator);

    println!("{:?}", res);
}

#[test]
fn test_integer_nth_root() {
    let y = 11100;
    let n = 3;

    let res = integer_nth_root(y, n);

    println!("{:?}", res);
}

fn generate_keccak(hash_b: &mut [u8; 136], nonce: &i128) -> [u8; 32] {
    let mut hash = [0u8; 32];

    hash_b[88..88 + 16].copy_from_slice(&nonce.to_be_bytes());

    let mut keccak = Keccak::v256();
    keccak.update(hash_b);
    keccak.finalize(&mut hash);

    hash
}

fn generate_hash(
    env: &Env,
    mine: &Address,
    miner: &Address,
    index: &u64,
    nonce: &i128,
    entropy: &BytesN<32>,
) -> [u8; 136] {
    let mut hash_b = [0u8; 136];

    let mut mine_b = [0u8; 40];
    mine.to_xdr(&env).copy_into_slice(&mut mine_b);

    let mut miner_b = [0u8; 40];
    miner.clone().to_xdr(&env).copy_into_slice(&mut miner_b);

    let index_b = index.to_be_bytes();
    let nonce_b = nonce.to_be_bytes();

    hash_b[0..40].copy_from_slice(&mine_b);
    hash_b[40..40 + 40].copy_from_slice(&miner_b);
    hash_b[80..80 + 8].copy_from_slice(&index_b);
    hash_b[88..88 + 16].copy_from_slice(&nonce_b);
    hash_b[104..104 + 32].copy_from_slice(&entropy.to_array());

    return hash_b;
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
