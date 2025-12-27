const DEFAULT_STYLE: &str = include_str!("../../../../assets/bright.json");

#[derive(uniffi::Object)]
pub struct MapState {
    style_json: String,
}

impl MapState {
    pub async fn new() -> Self {
        Self {
            style_json: DEFAULT_STYLE.to_string(),
        }
    }
}

#[uniffi::export]
impl MapState {
    pub fn get_style(&self) -> String {
        self.style_json.clone()
    }
}
