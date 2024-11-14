use askgql::{
    gpt::{generate_query, generate_response},
    json_to_schema::json_to_schema,
};
use clap::Parser;

/// A simple CLI to interact with a GraphQL server.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL of the GraphQL server.
    #[arg(short, long)]
    url: String,

    /// The inquiry to send to the GraphQL server.
    #[arg(short, long)]
    inquiry: String,

    /// OpenAI API key.
    #[arg(short, long)]
    api_key: String,

    /// The authorization token to send to the GraphQL server.
    #[arg(long)]
    authorization: Option<String>,

    /// The language to use for the response.
    #[arg(short, long)]
    language: Option<String>,

    /// Omit the comments from the GraphQL schema.
    #[arg(long)]
    omit_schema_comments: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("api server url: {}", args.url);

    let client = reqwest::Client::new();
    let gptcl = gptcl::GptClient::new(gptcl_hyper::HyperClient::new(), args.api_key.to_owned());

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("askgql"),
    );
    if let Some(auth) = &args.authorization {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(auth).unwrap(),
        );
    }

    let body = INTROSPECTION_QUERY.to_owned();
    let res = client
        .post(&args.url)
        .headers(headers.clone())
        .body(body)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let schema = String::from_utf8_lossy(&res);
    println!("schema (json) size: {}", schema.len());

    let schema = json_to_schema(&schema, !args.omit_schema_comments);
    println!("schema size: {}", schema.len());
    // println!("schema: {}", schema);
    // return;

    println!("inquiry: {}", args.inquiry);

    let query = generate_query(&gptcl, &schema, &args.inquiry)
        .await
        .unwrap();
    println!("gql query: {:?}", query);

    let body = serde_json::json!({
        // "operationName": "",
        "variables": {},
        "query": query
    })
    .to_string();

    let res = client
        .post(&args.url)
        .headers(headers.clone())
        .body(body)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let res = String::from_utf8_lossy(&res);
    println!("gql response: {}", res);

    let response = generate_response(&gptcl, &query, &res, &args.language)
        .await
        .unwrap();
    println!("response: {}", response);
}

const INTROSPECTION_QUERY: &str = r#"
{"operationName":"IntrospectionQuery","variables":{},"query":"query IntrospectionQuery {\n  __schema {\n    queryType {\n      name\n    }\n    mutationType {\n      name\n    }\n    subscriptionType {\n      name\n    }\n    types {\n      ...FullType\n    }\n    directives {\n      name\n      description\n      locations\n      args {\n        ...InputValue\n      }\n    }\n  }\n}\n\nfragment FullType on __Type {\n  kind\n  name\n  description\n  fields(includeDeprecated: true) {\n    name\n    description\n    args {\n      ...InputValue\n    }\n    type {\n      ...TypeRef\n    }\n    isDeprecated\n    deprecationReason\n  }\n  inputFields {\n    ...InputValue\n  }\n  interfaces {\n    ...TypeRef\n  }\n  enumValues(includeDeprecated: true) {\n    name\n    description\n    isDeprecated\n    deprecationReason\n  }\n  possibleTypes {\n    ...TypeRef\n  }\n}\n\nfragment InputValue on __InputValue {\n  name\n  description\n  type {\n    ...TypeRef\n  }\n  defaultValue\n}\n\nfragment TypeRef on __Type {\n  kind\n  name\n  ofType {\n    kind\n    name\n    ofType {\n      kind\n      name\n      ofType {\n        kind\n        name\n        ofType {\n          kind\n          name\n          ofType {\n            kind\n            name\n            ofType {\n              kind\n              name\n              ofType {\n                kind\n                name\n              }\n            }\n          }\n        }\n      }\n    }\n  }\n}\n"}
"#;
