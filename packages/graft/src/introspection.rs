use eyre::Result;
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};

use crate::{app, cross, graphql::schema::Schema, print_info};

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
pub struct Response {
    pub data: Data,
}

impl Response {
    pub async fn fetch(ctx: &app::Context, url: &str, no_ssl: bool) -> Result<Self> {
        let body = IntrospectionQuery::build_query(introspection_query::Variables {});

        let json = cross::net::fetch_json(url, no_ssl, body).await?;

        print_info!(ctx, 3, "Recieved json: {}", json);

        Ok(serde_json::from_value(json)?)
    }

    pub fn schema(self) -> Schema {
        self.data.schema
    }
}
