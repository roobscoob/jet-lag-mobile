use std::{
    fs::{self, File},
    hash::Hasher,
    io::Write,
    path::{Path, PathBuf},
};

use futures_util::StreamExt;

use crate::resource::{
    fetcher::{FetchError, ResourceFetcher},
    reference::ResourceReference,
};

pub struct ResourceStore {
    base_path: PathBuf,
    fetcher: ResourceFetcher,
}

impl ResourceStore {
    pub fn new(base_path: impl Into<PathBuf>, fetcher: ResourceFetcher) -> std::io::Result<Self> {
        let base_path = base_path.into();
        fs::create_dir_all(&base_path)?;

        Ok(Self { base_path, fetcher })
    }

    /// Get the path where a resource is stored
    pub fn resource_path(&self, reference: &ResourceReference) -> PathBuf {
        self.base_path.join(format!("{}", reference.id()))
    }

    /// Get the base path of the store
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Check if a resource exists locally
    pub fn contains(&self, reference: &ResourceReference) -> bool {
        self.resource_path(reference).exists()
    }

    /// Check if a resource exists locally by reference
    pub fn contains_ref(&self, reference: &ResourceReference) -> bool {
        self.contains(reference)
    }

    /// Get a resource, fetching it if not present
    pub async fn get(&mut self, reference: &ResourceReference) -> Result<PathBuf, StoreError> {
        let path = self.resource_path(reference);

        if !path.exists() {
            self.fetch_to_disk(reference).await?;
        }

        Ok(path)
    }

    /// Open a file handle to a resource, fetching if not present
    pub async fn open(&mut self, reference: &ResourceReference) -> Result<File, StoreError> {
        let path = self.get(reference).await?;
        Ok(File::open(&path)?)
    }

    /// Read a resource fully into memory, fetching if not present
    pub async fn read(&mut self, reference: &ResourceReference) -> Result<Vec<u8>, StoreError> {
        let path = self.get(reference).await?;
        Ok(fs::read(&path)?)
    }

    /// Remove a resource from disk
    pub fn remove(&mut self, reference: &ResourceReference) -> std::io::Result<()> {
        let path = self.resource_path(reference);

        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }

    /// Get the fetcher for direct bundle fetching
    pub fn fetcher(&self) -> &ResourceFetcher {
        &self.fetcher
    }

    /// Fetch a resource from the network and write it to disk
    async fn fetch_to_disk(&self, reference: &ResourceReference) -> Result<(), StoreError> {
        let path = self.resource_path(reference);
        let mut stream = self.fetcher.fetch_resource(reference).await?;

        let mut file = File::create(&path)?;
        let mut hasher = twox_hash::xxhash3_64::Hasher::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes)?;
            hasher.write(&bytes);
        }

        let computed_hash = hasher.finish();

        if computed_hash != reference.hash() {
            fs::remove_file(&path)?;
            return Err(StoreError::Fetch(FetchError::InvalidData(
                "Hash mismatch".to_string(),
            )));
        }

        file.sync_all()?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Fetch(FetchError),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "IO error: {}", e),
            StoreError::Fetch(e) => write!(f, "Fetch error: {}", e),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            StoreError::Fetch(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<FetchError> for StoreError {
    fn from(e: FetchError) -> Self {
        StoreError::Fetch(e)
    }
}
