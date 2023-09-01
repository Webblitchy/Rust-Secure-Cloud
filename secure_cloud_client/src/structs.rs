use dryoc::dryocsecretbox::{DryocSecretBox, Mac, Nonce};
use dryoc::constants::{CRYPTO_PWHASH_SALTBYTES};
use dryoc::classic::crypto_secretbox::{Key as CryptoKey};
use serde::{Deserialize, Serialize};


pub type Key = CryptoKey;
pub type Salt = [u8; CRYPTO_PWHASH_SALTBYTES]; // [u8; 16]


#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedBox(
    pub DryocSecretBox<Mac, Vec<u8>>,
    pub Nonce // StackByteArray<24: usize>
);


#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub encrypted_shard: EncryptedBox,
    pub salt: Salt // [u8; 16]
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Company {
    pub name: String,
    pub users: Vec<User>,
    pub masterkey_encrypted: EncryptedBox,
    pub hmackey: Key, // [u8; 32]
    pub hmackey_encrypted: EncryptedBox,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileNameBox(
    pub String, // UUID
    pub EncryptedBox // encrypted name
);

#[derive(Clone)]
pub enum RequestType {
    CloseConnexion,
    CreateCompany,
    AuthenticateSession,
    UploadFile,
    GetFilenames,
    DownloadFile,
    RegenerateKey
}
