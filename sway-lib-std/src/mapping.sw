library mapping;

use ::hash::{HashMethod, hash_pair, hash_u64, hash_value};
use core::ops::*;

pub struct Mapping {
    seed_key: b256
}

impl Mapping {
    fn insert(self, key: u64, value: u64) {
        let key_hash = hash_u64(key, HashMethod::Sha256);
        let hash_with_seed = hash_pair(key_hash, self.seed_key, HashMethod::Sha256);
        value.write(key_hash);
    }

    fn get(self, key: u64) -> u64 {
        let key_hash = hash_u64(key, HashMethod::Sha256);
        let hash_with_seed = hash_pair(key_hash, self.seed_key, HashMethod::Sha256);
        ~u64::read(key_hash)
    }
}
