use crate::shamir::*;
use crate::structs::*;
use crate::tui::Interface;
use crate::tui::PopupType;
use crate::{crypto::*, tui};
use dryoc::classic::crypto_secretbox::{crypto_secretbox_keygen, Key};
use dryoc::rng::copy_randombytes;
use shamirsecretsharing::DATA_SIZE;

fn create_users(grouped_shards: &[u8; DATA_SIZE], interface: &mut Interface<'_>) -> Vec<User> {
    // let nb_users = input_nb_users();
    let nb_users: u8 = tui::input_field(interface, "User number", &ValidationType::NbMinUser)
        .ok()
        .unwrap()
        .parse()
        .unwrap();

    let shards = create_shards(grouped_shards, nb_users);

    let mut users: Vec<User> = Vec::new();
    let mut i = 0;
    while i < nb_users as usize {
        let (username, password) = tui::user_passwd_input(interface, i + 1, true).ok().unwrap();
        let mut already_taken = false;
        for u in &users {
            if u.username == username {
                interface.set_popup("Username already taken !", PopupType::Error);
                already_taken = true;
                break;
            }
        }
        if already_taken {
            continue;
        }

        let (encrypted_shard, salt) = encrypt_shard(&password, &shards[i]);
        let user = User {
            username,
            encrypted_shard,
            salt,
        };
        users.push(user);
        i += 1;
    }
    users
}

pub fn create_company(term: &mut Interface<'_>) -> Company {
    let master_key = crypto_secretbox_keygen() as Key; // u8[32]
    let hmackey = crypto_secretbox_keygen() as Key; // u8[32]

    let company_name = tui::input_field(term, "Company name", &ValidationType::NotEmpty)
        .ok()
        .unwrap();
    rekey_company(&master_key, &hmackey, &company_name, term)
}

pub fn rekey_company(
    masterkey: &Key,
    hmackey: &Key,
    company_name: &String,
    term: &mut Interface<'_>,
) -> Company {
    let mut grouped_shards = [0u8; DATA_SIZE];
    copy_randombytes(&mut grouped_shards);

    let group_key = generate_group_key(&grouped_shards);

    let hmackey_encrypted = encrypt(&hmackey.to_vec(), &group_key);

    let users = create_users(&grouped_shards, term);

    let masterkey_encrypted = encrypt(&masterkey.to_vec(), &group_key);

    Company {
        name: company_name.clone(),
        users,
        masterkey_encrypted,
        hmackey: hmackey.clone(),
        hmackey_encrypted,
    }
}
