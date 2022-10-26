use eyre::Result;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct AppConfig {
    pub schema: Url,
    pub no_ssl: Option<bool>,
}

pub fn load() -> Result<AppConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("g2t-config"))
        .build()?
        .try_deserialize::<AppConfig>()
}
