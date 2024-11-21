pub mod gql;
pub mod json_to_schema;

use std::io::{BufRead, Write};

pub type GptClient = gptcl::GptClient<gptcl_hyper::HyperClient>;

pub async fn process_interactive(
    gql: gql::GqlClient,
    gpt: &GptClient,
    schema: String,
    first_message: Option<String>,
) {
    let function_name = "graphQLRequest";

    let mut req = gpt_model::ChatRequest::from_model(gptcl::MODEL_GPT_4O_MINI.to_owned());
    req.temperature = Some(0.0);
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

    req.messages
        .push(gpt_model::ChatMessage::from_system(format!(
            r#"You are an assistant that can interact with a GraphQL server.

GraphQL schema:
```
{}
```"#,
            schema
        )));
    if let Some(message) = first_message {
        req.messages
            .push(gpt_model::ChatMessage::from_user(message));
    }

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
                print!("user > ");
                std::io::stdout().flush().unwrap();
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
