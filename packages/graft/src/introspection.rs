use eyre::{Result, eyre};
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};

use crate::{app, cross, graphql::schema::Schema, print_info, cross_eprintln};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/introspection_schema.graphql",
    query_path = "src/graphql/introspection_query.graphql",
    response_derives = "Serialize",
    variable_derives = "Deserialize"
)]
struct IntrospectionQuery;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Successful {
        data: Data,
    },
    Error {
        data: Option<Data>,
        errors: serde_json::Value,
    }
}

impl Response {
    pub async fn fetch(ctx: &app::Context, url: &str, no_ssl: bool) -> Result<Self> {
        let body = IntrospectionQuery::build_query(introspection_query::Variables {});

        let json = cross::net::fetch_json(url, no_ssl, body).await?;

        print_info!(ctx, 3, "Recieved json: {}", json);

        Ok(serde_json::from_value(json)?)
    }

    pub fn schema(self) -> Result<Schema> {
        match self {
            Self::Successful { data } => {
                Ok(data.schema)
            },
            Self::Error { data: Some(data), errors } => {
                // FIXME: Need some proper logging. Maybe tracing?
                cross_eprintln!("{}", console::style(format!("GraphQL data came with errors: {errors}")).yellow());
                Ok(data.schema)
            }
            Self::Error { data: None, errors } => {
                Err(eyre!("GraphQL error: {errors}"))
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::{app, Response};

    #[tokio::test]
    async fn schema_json_output() {
        let ctx = app::Context {
            verbose: 0,
            config_location: None,
        };

        let schema = Response::fetch(
            &ctx,
            "https://swapi-graphql.netlify.app/.netlify/functions/index",
            false,
        )
        .await
        .expect("schema fetch error")
        .schema()
        .unwrap();

        insta::assert_snapshot!(serde_json::to_string_pretty(&schema).expect("fail to pretty string schema"));
    }
}
