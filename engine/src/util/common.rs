use sha1::{Digest, Sha1};


pub fn sha1_hash(bytes: impl AsRef<[u8]>) -> [u8; 20] {
    let mut hash = [0u8; 20];
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let hasher_output = hasher.finalize();
    let result: &[u8] = hasher_output.as_slice();
    hash.copy_from_slice(result);
    hash
}