use dryoc::dryocsecretbox::{DryocSecretBox, Mac, NewByteArray, Nonce};
use dryoc::constants::{CRYPTO_PWHASH_SALTBYTES};
use dryoc::classic::crypto_secretbox::{Key as CryptoKey};
use serde::{Deserialize, Serialize};
use num_enum::TryFromPrimitive;



pub type Key = CryptoKey;
pub type Salt = [u8; CRYPTO_PWHASH_SALTBYTES];

#[derive(Clone)]
#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedBox(
    pub DryocSecretBox<Mac, Vec<u8>>,
    pub Nonce // StackByteArray<24: usize>
);

#[derive(Clone)]
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub encrypted_shard: EncryptedBox,
    pub salt: Salt // [u8; 16]
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Company {
    pub name: String,
    pub users: Vec<User>,
    pub masterkey_encrypted: EncryptedBox,
    pub hmackey: Key, // [u8; 32]
    pub hmackey_encrypted: EncryptedBox,
}

impl Company {
    pub fn find_user(&self, user_to_find: String) -> Option<User> {
        for user in &self.users {
            if user.username == user_to_find {
                return Some(user.clone())
            }
        }
        None
    }
}

impl Company {
    pub fn empty_company() -> Company {
        Company {
            name: "".to_string(),
            users: vec![],
            masterkey_encrypted: EncryptedBox(
                DryocSecretBox::from_parts(Mac::gen(),vec![]),
                Default::default()
            ),
            hmackey: [0; 32],
            hmackey_encrypted: EncryptedBox(
                DryocSecretBox::from_parts(Mac::gen(),vec![]),
                Default::default()
            )
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileNameBox(
    pub String, // UUID
    pub EncryptedBox
);

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum RequestType {
    CloseConnexion,
    CreateCompany,
    AuthenticateSession,
    SaveFile,
    GetFilenames,
    SendFile,
    RegenerateKey
}
