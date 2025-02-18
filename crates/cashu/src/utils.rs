//! Utils

use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::Hash;
use rand::prelude::*;
use regex::Regex;

pub fn extract_url_from_error(error: &str) -> Option<String> {
    let regex = Regex::new(r"https?://[^\s]+").unwrap();
    if let Some(capture) = regex.captures(error) {
        return Some(capture[0].to_owned());
    }
    None
}

pub fn random_hash() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; Sha256::LEN];
    rng.fill_bytes(&mut random_bytes);
    let hash = Sha256::hash(&random_bytes);
    hash.to_byte_array().to_vec()
}
