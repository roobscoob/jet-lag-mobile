use std::{hash::Hasher, io::Read, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    version: u64,
    size: u64,
    xxhash: u64,
    id: Arc<str>,
}

impl ResourceReference {
    pub fn new(id: impl Into<Arc<str>>, version: u64, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let metadata = std::fs::metadata(&path).expect("Failed to read file metadata");
        let size = metadata.len();

        let mut file = std::fs::File::open(&path).expect("Failed to open file for hashing");
        let mut hasher = twox_hash::xxhash3_64::Hasher::new();

        loop {
            let mut buffer = [0u8; 8192];
            let n = file.read(&mut buffer).expect("Failed to read file chunk");
            if n == 0 {
                break;
            }
            hasher.write(&buffer[..n]);
        }

        let xxhash = hasher.finish();

        Self {
            id: id.into(),
            version,
            size,
            xxhash,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn hash(&self) -> u64 {
        self.xxhash
    }
}
