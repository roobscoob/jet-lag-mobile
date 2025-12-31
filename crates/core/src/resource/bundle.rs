use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::resource::reference::ResourceReference;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBundle {
    id: Arc<str>,
    version: u64,
    size: u64,
    resources: HashMap<String, ResourceReference>,
}

impl ResourceBundle {
    pub fn new(id: impl Into<Arc<str>>, version: u64) -> Self {
        Self {
            id: id.into(),
            version,
            size: 0,
            resources: HashMap::new(),
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

    pub fn resources(&self) -> &HashMap<String, ResourceReference> {
        &self.resources
    }

    pub fn get(&self, name: &str) -> Option<&ResourceReference> {
        self.resources.get(name)
    }

    pub fn insert(&mut self, name: String, resource: ResourceReference) {
        self.size += resource.size();
        self.resources.insert(name, resource);
    }

    pub fn remove(&mut self, name: &str) -> Option<ResourceReference> {
        let resource = self.resources.remove(name)?;
        self.size -= resource.size();
        Some(resource)
    }
}
