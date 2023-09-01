use crate::structs::*;
use crate::crypto::*;
use shamirsecretsharing::{combine_shares, create_shares, DATA_SIZE};
use dryoc::constants::{CRYPTO_PWHASH_SALTBYTES};
use dryoc::Error;
use dryoc::rng::{copy_randombytes};


pub fn create_shards(grouped_shards: &[u8; DATA_SIZE], nb_users: u8) -> Vec<Vec<u8>>{
    create_shares(grouped_shards, nb_users, 2).unwrap()
}

pub fn encrypt_shard(password: &str, shard: &Vec<u8>) -> (EncryptedBox, Salt) {

    let mut salt = [0u8; CRYPTO_PWHASH_SALTBYTES];
    copy_randombytes(&mut salt);

    let secret_key = key_derivation(password, &salt);

    (encrypt(shard, &secret_key), salt)
}

pub fn decrypt_shard(password: &str, encrypted_shard: &EncryptedBox, salt: &Salt) -> Result<Vec<u8>, Error> {
    let secret_key = key_derivation(password, salt);

    decrypt(encrypted_shard, &secret_key)
}

pub fn rebuild_grouped_shards(shards: Vec<Vec<u8>>) -> Option<[u8;64]> {
    match combine_shares(&shards).unwrap() {
        Some(combined_shares) => {
            let combined_shares: [u8; 64] = combined_shares.try_into().unwrap();
            Some(combined_shares)
        }
        None => None
    }
}