use serde::Serialize;

const CONNECTION_TYPE: &str = "stdio";
const NATIVE_APP_NAME: &str = "external_editor_revived";
const EXTENSION_ID: &str = "external-editor-revived@tsundere.moe";

#[derive(Debug, Serialize)]
pub struct AppManifest {
    pub name: &'static str,
    pub description: &'static str,
    pub path: String,
    #[serde(rename = "type")]
    pub connection_type: &'static str,
    pub allowed_extensions: Vec<&'static str>,
}

impl AppManifest {
    pub fn new(path: &str) -> Self {
        Self {
            name: NATIVE_APP_NAME,
            description: env!("CARGO_PKG_DESCRIPTION"),
            path: path.to_owned(),
            connection_type: CONNECTION_TYPE,
            allowed_extensions: vec![EXTENSION_ID],
        }
    }
}
