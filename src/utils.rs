use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn uuid_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:?}", Uuid::new_v5(&Uuid::NAMESPACE_OID, &result[..16]))
}
