use config::{Config, ConfigError};
use eyre::Result;
use serde::Deserialize;
use url::Url;

use crate::util::path_with_possible_prefix;

#[derive(Deserialize)]
pub struct AppConfig {
    pub schema: Url,
    pub no_ssl: Option<bool>,
    pub document: String,
}

pub fn load(dir: Option<&str>) -> Result<AppConfig, ConfigError> {
    let config_name = ".accord";

    let path = path_with_possible_prefix(dir, config_name);

    Config::builder()
        .add_source(config::File::with_name(path.to_str().ok_or_else(|| {
            ConfigError::Message("Config path wasn't valid unicode.".to_owned())
        })?))
        .build()?
        .try_deserialize::<AppConfig>()
}
