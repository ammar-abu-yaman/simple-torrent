use sha1::{Digest, Sha1};
use crate::model::Sha1Hash;

pub fn sha1_hash(bytes: impl AsRef<[u8]>) -> Sha1Hash {
    let mut hash = [0u8; 20];
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let hasher_output = hasher.finalize();
    let result: &[u8] = hasher_output.as_slice();
    hash.copy_from_slice(result);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let bytes = b"hello world";
        let correct_hash = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        assert_eq!(hex::encode(sha1_hash(bytes)), correct_hash);
    }
}