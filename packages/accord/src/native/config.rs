use config::{Config, ConfigError};
use eyre::Result;

pub use crate::config::RawAppConfig;
use crate::util::path_with_possible_prefix;

pub fn load(dir: Option<&str>) -> Result<RawAppConfig, ConfigError> {
    let config_name = ".accord";

    let path = path_with_possible_prefix(dir, config_name);

    Config::builder()
        .add_source(config::File::with_name(path.to_str().ok_or_else(|| {
            ConfigError::Message("Config path wasn't valid unicode.".to_owned())
        })?))
        .build()?
        .try_deserialize::<RawAppConfig>()
}
