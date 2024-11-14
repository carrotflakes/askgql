pub mod gpt;
pub mod gql;
pub mod json_to_schema;

use std::io::BufRead;

use gpt::{generate_query, generate_response};

pub type GptClient = gptcl::GptClient<gptcl_hyper::HyperClient>;

pub async fn process_inquiry(
    gql: gql::GqlClient,
    gpt: &GptClient,
    schema: String,
    inquiry: &str,
    language: &Option<String>,
) {
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

pub async fn process_interactive(gql: gql::GqlClient, gpt: &GptClient, schema: String) {
    let function_name = "graphQLRequest";

    let mut req = gpt_model::ChatRequest::from_model(gptcl::MODEL_GPT_4O_MINI.to_owned());
    req.temperature = Some(0.0);
    req.messages = vec![gpt_model::ChatMessage::from_system(format!(
        r#"You have a GraphQL server and an assistant for it.

GraphQL schema:
```
{}
```"#,
        schema
    ))];
    req.functions = std::sync::Arc::new(vec![gpt_model::Function {
        name: function_name.to_owned(),
        description: Some(format!("Send GraphQL request and receive the response")),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": r#"GraphQL query like `{"query": "query {user {name email}}"}`"#,
                }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
    }]);

    loop {
        let res = gpt.call(&req).await.unwrap();
        req.messages.push(res.choices[0].message.clone());

        match res.choices[0].finish_reason.as_str() {
            "stop" => {
                let Some(mes) = &res.choices[0].message.content else {
                    println!("no message");
                    return;
                };
                println!("ai > {}", mes);
                println!("user >");
                let mut input = String::new();
                std::io::stdin().lock().read_line(&mut input).unwrap();
                req.messages
                    .push(gpt_model::ChatMessage::from_user(input.trim().to_owned()));
            }
            "function_call" => {
                let Some(fc) = &res.choices[0].message.function_call else {
                    println!("no function call");
                    return;
                };
                if fc.name == function_name {
                    let body = fc.arguments.clone();
                    println!("gql query: {}", &body);
                    let res = gql.request(body).await.unwrap();
                    println!("gql response: {}", res);
                    req.messages
                        .push(gpt_model::ChatMessage::from_function_response(
                            function_name.to_owned(),
                            res,
                        ));
                } else {
                    println!("unknown function: {}", fc.name);
                }
            }
            finish_reason => {
                println!("finish reason: {}", finish_reason);
                return;
            }
        }
    }
}
