pub struct FieldEnumVariant {
    pub identifier: String,
    pub display_name: String,
}

pub enum Field {
    Text,
    Enum { variants: Vec<FieldEnumVariant> },
}
