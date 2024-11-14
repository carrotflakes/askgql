use crate::json_to_schema::json_to_schema;

pub struct GqlClient {
    client: reqwest::Client,
    url: String,
    headers: reqwest::header::HeaderMap,
}

impl GqlClient {
    pub fn new(url: String, authorization: &Option<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("askgql"),
        );
        if let Some(auth) = authorization {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(auth).unwrap(),
            );
        }
        let client = reqwest::Client::new();
        Self {
            client,
            url,
            headers,
        }
    }

    pub async fn introspect(&self, omit_schema_comments: bool) -> Result<String, String> {
        let body = INTROSPECTION_QUERY.to_owned();
        let schema = self.request(body).await?;
        println!("schema (json) size: {}", schema.len());

        let schema = json_to_schema(&schema, !omit_schema_comments);
        println!("schema size: {}", schema.len());
        Ok(schema)
    }

    pub async fn request(&self, body: String) -> Result<String, String> {
        let res = self
            .client
            .post(&self.url)
            .headers(self.headers.clone())
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&res).to_string())
    }
}

const INTROSPECTION_QUERY: &str = r#"
{"operationName":"IntrospectionQuery","variables":{},"query":"query IntrospectionQuery {\n  __schema {\n    queryType {\n      name\n    }\n    mutationType {\n      name\n    }\n    subscriptionType {\n      name\n    }\n    types {\n      ...FullType\n    }\n    directives {\n      name\n      description\n      locations\n      args {\n        ...InputValue\n      }\n    }\n  }\n}\n\nfragment FullType on __Type {\n  kind\n  name\n  description\n  fields(includeDeprecated: true) {\n    name\n    description\n    args {\n      ...InputValue\n    }\n    type {\n      ...TypeRef\n    }\n    isDeprecated\n    deprecationReason\n  }\n  inputFields {\n    ...InputValue\n  }\n  interfaces {\n    ...TypeRef\n  }\n  enumValues(includeDeprecated: true) {\n    name\n    description\n    isDeprecated\n    deprecationReason\n  }\n  possibleTypes {\n    ...TypeRef\n  }\n}\n\nfragment InputValue on __InputValue {\n  name\n  description\n  type {\n    ...TypeRef\n  }\n  defaultValue\n}\n\nfragment TypeRef on __Type {\n  kind\n  name\n  ofType {\n    kind\n    name\n    ofType {\n      kind\n      name\n      ofType {\n        kind\n        name\n        ofType {\n          kind\n          name\n          ofType {\n            kind\n            name\n            ofType {\n              kind\n              name\n              ofType {\n                kind\n                name\n              }\n            }\n          }\n        }\n      }\n    }\n  }\n}\n"}
"#;
