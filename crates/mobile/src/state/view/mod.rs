use std::sync::Arc;

use tokio::sync::RwLock;

use crate::state::view::map::MapState;

pub mod map;

#[derive(uniffi::Object)]
pub struct ViewState {
    map: RwLock<Option<Arc<MapState>>>,
}

#[uniffi::export]
impl ViewState {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            map: RwLock::new(None),
        }
    }

    pub async fn get_map_state(&self) -> Arc<MapState> {
        if let Some(ref map) = *(self.map.read().await) {
            return Arc::clone(map);
        }

        let mut guard = self.map.write().await;
        let new_map = Arc::new(MapState::new().await);
        *guard = Some(Arc::clone(&new_map));

        new_map
    }
}
