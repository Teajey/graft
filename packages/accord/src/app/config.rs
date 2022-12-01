use std::path::PathBuf;

use eyre::eyre;
use eyre::{Report, Result};
use regex::Captures;
use regex_macro::regex;
use serde::Deserialize;
use url::Url;

use crate::{cross, util};

#[derive(Deserialize)]
pub struct RawAppConfig {
    pub schema: String,
    pub no_ssl: Option<bool>,
    pub document: Option<PathBuf>,
    pub emit_schema: Option<bool>,
}

pub struct Config {
    pub schema: Url,
    pub no_ssl: bool,
    pub document_path: Option<String>,
    pub emit_schema: bool,
}

impl TryFrom<RawAppConfig> for Config {
    type Error = Report;

    fn try_from(raw: RawAppConfig) -> Result<Self> {
        let envvar_interpolator = regex!(r#"\{\{(\w+)\}\}"#);

        // FIXME: This' pretty hacky, but I can't think of a better way to deal with `std::env::var`s `Result` inside `replace_all` right now
        const ENVVAR_NOT_FOUND: &str = "{{ENVVAR NOT FOUND}}";

        let schema = envvar_interpolator.replace_all(&raw.schema, |captures: &Captures<'_>| {
            let envvar_key = captures.get(1).expect("first capture defined in envvar_interpolator");
            let envvar_key = envvar_key.as_str();
            cross::env::var(envvar_key).unwrap_or_else(|_| {
                eprintln!("Couldn't find environment variable with name \"{}\" while interpolating schema", envvar_key);
                ENVVAR_NOT_FOUND.to_owned()
            })
        });

        if schema.contains(ENVVAR_NOT_FOUND) {
            return Err(eyre!("Missing environment variable"));
        }

        let schema = Url::try_from(schema.as_ref())?;

        let document_path = match raw.document {
            Some(document) => Some(document.into_os_string().into_string().map_err(
                |os_string| eyre!("Failed to parse document path as UTF-8: {os_string:?}"),
            )?),
            None => None,
        };

        Ok(Config {
            schema,
            no_ssl: raw.no_ssl.unwrap_or(false),
            document_path,
            emit_schema: raw.emit_schema.unwrap_or(false),
        })
    }
}

impl Config {
    pub fn load(dir: Option<&str>) -> Result<Self> {
        let config_name = ".accord.yml";

        let path = util::path_with_possible_prefix(dir, config_name);

        let config_string = cross::fs::read_to_string(path)?;

        let config: RawAppConfig = serde_yaml::from_str(&config_string)?;

        config.try_into()
    }
}
