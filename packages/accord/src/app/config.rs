use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eyre::Result;
use regex::Captures;
use regex_macro::regex;
use serde::{de::Visitor, Deserialize};
use url::Url;

use crate::{cross, util};

#[derive(Debug)]
pub struct Glob(pub Vec<PathBuf>);

struct GlobVisitor;

impl<'de> Visitor<'de> for GlobVisitor {
    type Value = Glob;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a glob path string")
    }

    fn visit_str<E>(self, st: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match crate::cross::fs::glob(st) {
            Ok(paths) => Ok(Glob(paths)),
            Err(err) => Err(E::custom(err)),
        }
    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(GlobVisitor)
    }
}

#[derive(Debug)]
pub struct EnvvarUrl(pub Url);

struct EnvvarUrlVisitor;

impl<'de> Visitor<'de> for EnvvarUrlVisitor {
    type Value = EnvvarUrl;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a url string possibly containing envvars for interpolation, e.g.: {{TOKEN}}",
        )
    }

    fn visit_str<E>(self, st: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let envvar_interpolator = regex!(r#"\{\{(\w+)\}\}"#);

        // FIXME: This' pretty hacky, but I can't think of a better way to deal with `std::env::var`s `Result` inside `replace_all` right now
        const ENVVAR_NOT_FOUND: &str = "{{ENVVAR NOT FOUND}}";

        let url = envvar_interpolator.replace_all(st, |captures: &Captures<'_>| {
            let envvar_key = captures.get(1).expect("first capture defined in envvar_interpolator");
            let envvar_key = envvar_key.as_str();
            cross::env::var(envvar_key).unwrap_or_else(|_| {
                eprintln!("Couldn't find environment variable with name \"{}\" while interpolating schema", envvar_key);
                ENVVAR_NOT_FOUND.to_owned()
            })
        });

        if url.contains(ENVVAR_NOT_FOUND) {
            return Err(E::custom("Missing environment variable"));
        }

        let url = Url::try_from(url.as_ref()).map_err(|err| E::custom(err))?;

        Ok(EnvvarUrl(url))
    }
}

impl<'de> Deserialize<'de> for EnvvarUrl {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(EnvvarUrlVisitor)
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SchemaGenOut {
    #[serde(rename = "ast")]
    pub ast_path: Option<PathBuf>,
    #[serde(rename = "json")]
    pub json_path: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SchemaGenPlan {
    pub url: EnvvarUrl,
    #[serde(default)]
    pub no_ssl: bool,
    pub out: SchemaGenOut,
}

#[derive(Deserialize, Debug)]
pub struct DocumentImport(String, String);

impl DocumentImport {
    pub fn type_name(&self) -> &str {
        &self.0
    }

    pub fn package(&self) -> &str {
        &self.1
    }
}

impl Default for DocumentImport {
    fn default() -> Self {
        Self("TypedQueryDocumentNode".to_owned(), "graphql".to_owned())
    }
}

mod default_options {
    pub fn selection_set_suffix() -> String {
        "SelectionSet".to_owned()
    }

    pub fn arguments_suffix() -> String {
        "Args".to_owned()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TypescriptOptions {
    #[serde(default)]
    pub document_import: DocumentImport,
    pub scalar_newtypes: Option<HashMap<String, String>>,
    #[serde(default)]
    pub documents_hide_operation_name: bool,
    #[serde(default = "default_options::selection_set_suffix")]
    pub selection_set_suffix: String,
    #[serde(default = "default_options::arguments_suffix")]
    pub arguments_suffix: String,
}

impl Default for TypescriptOptions {
    fn default() -> Self {
        Self {
            document_import: DocumentImport::default(),
            scalar_newtypes: None,
            documents_hide_operation_name: Default::default(),
            selection_set_suffix: default_options::selection_set_suffix(),
            arguments_suffix: default_options::arguments_suffix(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TypescriptGenPlan {
    pub ast: PathBuf,
    pub documents: Option<Glob>,
    pub out: PathBuf,
    #[serde(default)]
    pub options: TypescriptOptions,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GenPlans {
    #[serde(rename = "schema")]
    pub schema_gen_plan: Option<SchemaGenPlan>,
    #[serde(rename = "typescript")]
    pub typescript_gen_plan: Option<TypescriptGenPlan>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub generates: HashMap<String, GenPlans>,
}

impl Config {
    pub fn load(dir: Option<&Path>) -> Result<Self> {
        let config_name = ".accord.yml";

        let path = util::path_with_possible_prefix(dir, &PathBuf::try_from(config_name)?);

        let config_string = cross::fs::read_to_string(path)?;

        Ok(serde_yaml::from_str(&config_string)?)
    }
}
