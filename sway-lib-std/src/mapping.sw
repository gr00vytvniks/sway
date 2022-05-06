library mapping;

use ::hash::{HashMethod, hash_pair, hash_u64, hash_value};
use ::chain::log_b256;
use core::ops::*;

pub struct Mapping {
    seed_key: b256
}

impl<V> Mapping {
    fn insert(self, key: u64, value: u64) {
        let key_hash = hash_u64(key, HashMethod::Sha256);
        let hash_with_seed = hash_pair(key_hash, self.seed_key, HashMethod::Sha256);
        log_b256(hash_with_seed);
        value.write(key_hash);
    }

    fn get(self, key: u64) -> u64 {
        let key_hash = hash_u64(key, HashMethod::Sha256);
        let hash_with_seed = hash_pair(key_hash, self.seed_key, HashMethod::Sha256);
        log_b256(hash_with_seed);
        ~u64::read(key_hash)
    }
}
