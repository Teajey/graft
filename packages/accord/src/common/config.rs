use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct AppConfig {
    pub schema: Url,
    pub no_ssl: Option<bool>,
    #[serde(rename = "document")]
    pub document_path: Option<String>,
}
