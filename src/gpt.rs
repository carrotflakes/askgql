use gpt_model::{ChatMessage, ChatRequest};

pub async fn generate_query<T: gptcl::http_client::HttpClient>(
    gptcl: &gptcl::GptClient<T>,
    schema: &str,
    query: &str,
) -> Result<String, String> {
    let mut req = ChatRequest::from_model(gptcl::MODEL_GPT_4O_MINI.to_owned());
    // req.functions = Arc::new(vec![gpt_model::Function {
    //     name: "graphQLRequest".to_owned(),
    //     description: Some(format!("Send GraphQL request and receive the response\n\nschema: ```{}```", SCHEMA)),
    //     parameters: serde_json::json!({
    //         "type": "object",
    //         "properties": {
    //             "query": {
    //                 "type": "string",
    //                 "description": "The GraphQL query."
    //             }
    //         },
    //         "required": ["query"],
    //         "additionalProperties": false
    //     }),
    // }]);
    req.response_format = Some(gpt_model::ResponseFormat::Json);
    req.temperature = Some(0.0);
    req.messages = vec![
        ChatMessage::from_system(
            r#"Your response will be in JSON format like `{"query": "query {user {name email}}"}`"#.to_owned(),
        ),
        ChatMessage::from_user(format!(
            r#"There is a GraphQL server and the schema is here:
```
{}
```

Construct a query that accomplishes the following:
```
{}
```"#,
            schema, query
        )),
    ];
    let res = gptcl.call(&req).await.unwrap();

    #[derive(serde::Deserialize)]
    struct Response {
        query: String,
    }

    let Some(mes) = res.choices.get(0).and_then(|x| x.message.content.clone()) else {
        println!("{:?}", res);
        return Err("No response".to_owned());
    };
    let query = match serde_json::from_str::<Response>(&mes).map(|x| x.query) {
        Ok(q) => q,
        Err(e) => {
            println!("{:?}", e);
            return Err("Failed to parse response".to_owned());
        }
    };
    Ok(query)
}

pub async fn generate_response<T: gptcl::http_client::HttpClient>(
    gptcl: &gptcl::GptClient<T>,
    query: &str,
    gql_response: &str,
    language: &Option<String>,
) -> Result<String, String> {
    let mut req = ChatRequest::from_model(gptcl::MODEL_GPT_4O_MINI.to_owned());
    req.temperature = Some(0.0);
    req.messages = vec![ChatMessage::from_user(format!(
        r#"
The user asked:

```
{}
```

The GraphQL server responded with:

```
{}
```

Please answer the user question.
No need to mention the graphql response.{}
"#,
        query,
        gql_response,
        match language {
            Some(lang) => format!("\n\nLanguage: {}", lang),
            None => "".to_owned(),
        }
    ))];
    let res = gptcl.call(&req).await.unwrap();

    res.choices
        .get(0)
        .and_then(|x| x.message.content.clone())
        .ok_or("No response".to_owned())
}
