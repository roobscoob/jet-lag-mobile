use std::sync::Arc;

use crate::{resource::bundle::ResourceBundle, transit::TransitProvider};

pub struct Map {
    id: Arc<str>,
    name: Arc<str>,
    geography: ResourceBundle,
    transit: Arc<dyn TransitProvider>,
}
