pub mod gpt;
pub mod gql;
pub mod json_to_schema;

use gpt::{generate_query, generate_response};

pub type GptClient = gptcl::GptClient<gptcl_hyper::HyperClient>;

pub async fn process_inquiry(
    gql: gql::GqlClient,
    gpt: &GptClient,
    inquiry: &str,
    language: &Option<String>,
    omit_schema_comments: bool,
) {
    let schema = gql.introspect(omit_schema_comments).await.unwrap();
    // println!("schema: {}", schema);
    // return;

    println!("inquiry: {}", inquiry);

    let query = generate_query(gpt, &schema, inquiry).await.unwrap();
    println!("gql query: {:?}", query);

    let body = serde_json::json!({
        // "operationName": "",
        "variables": {},
        "query": query
    })
    .to_string();

    let res = gql.request(body).await.unwrap();
    println!("gql response: {}", res);

    let response = generate_response(gpt, &query, &res, language)
        .await
        .unwrap();
    println!("response: {}", response);
}
