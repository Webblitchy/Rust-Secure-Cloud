use crate::crypto::generate_group_key;
use crate::shamir::{decrypt_shard, rebuild_grouped_shards};
use crate::structs::*;

pub fn build_groupkey(creds: Vec<(&User, &str)>) -> Option<Key> {
    let mut shards = Vec::new();
    for (user, password) in creds {
        shards.push(
            match decrypt_shard(password, &user.encrypted_shard, &user.salt) {
                Ok(shard) => shard,
                Err(_) => {
                    eprintln!("Bad company / usernames / passwords");
                    return None;
                }
            },
        );
    }
    let grouped_shards = rebuild_grouped_shards(shards);

    if let Some(grouped_shards) = grouped_shards {
        Some(generate_group_key(&grouped_shards))
    } else {
        eprintln!("Bad company / usernames / passwords");
        None
    }
}

