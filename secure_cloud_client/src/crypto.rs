use crate::structs::*;

use dryoc::dryocsecretbox::{DryocSecretBox, NewByteArray, Nonce};
use dryoc::generichash::GenericHash;
use dryoc::pwhash::{Config, PwHash};
use dryoc::Error;
use p256::pkcs8::der::Encode;
use shamirsecretsharing::DATA_SIZE;

fn hash(input: &Vec<u8>) -> Vec<u8> {
    GenericHash::hash_with_defaults_to_vec::<_, Key>(input, None).expect("hash failed")
}

pub fn key_derivation(password: &str, salt: &Salt) -> Key {
    #[cfg(debug_assertions)]
    let config = Config::interactive(); // to test faster

    #[cfg(not(debug_assertions))]
    let config = Config::moderate();

    let key: Vec<u8> = PwHash::hash_with_salt(&password.as_bytes(), salt, config)
        .expect("pwhash failed")
        .into_parts()
        .0;

    Key::try_from(key).unwrap()
}

pub fn encrypt(data: &Vec<u8>, key: &Key) -> EncryptedBox {
    let nonce = Nonce::gen();

    let encrypted_data = DryocSecretBox::encrypt_to_vecbox(data, &nonce, key);

    EncryptedBox(encrypted_data, nonce)
}

pub fn decrypt(encryted_data: &EncryptedBox, key: &Key) -> Result<Vec<u8>, Error> {
    encryted_data.0.decrypt_to_vec(&encryted_data.1, key)
}

pub fn generate_group_key(grouped_shards: &[u8; DATA_SIZE]) -> Key {
    hash(&grouped_shards.to_vec().unwrap()).try_into().unwrap() // cannot panic
}
