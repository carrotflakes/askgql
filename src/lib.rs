pub mod gql;
pub mod json_to_schema;

use std::io::{BufRead, Write};

use gptcl::model as gpt_model;

pub type GptClient = gptcl::GptClient<gptcl_hyper::HyperClient>;

pub async fn process_interactive(
    gql: gql::GqlClient,
    gpt: &GptClient,
    model: String,
    schema: String,
    first_message: Option<String>,
) {
    let function_name = "graphQLRequest";

    let mut req = gpt_model::ChatRequest::from_model(model);
    req.temperature = Some(0.0);
    req.tools = std::sync::Arc::new(vec![gpt_model::Tool::Function {
        function: gpt_model::Function {
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
            strict: true,
        },
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
            "tool_calls" => {
                for tool_call in &res.choices[0].message.tool_calls {
                    if tool_call.r#type == "function" && tool_call.function.name == function_name {
                        let body = tool_call.function.arguments.clone();
                        println!("gql query: {}", &body);
                        let res = gql.request(body).await.unwrap();
                        println!("gql response: {}", res);
                        req.messages
                            .push(gpt_model::ChatMessage::from_tool_response(
                                tool_call.id.clone(),
                                res,
                            ));
                    } else {
                        println!("unknown tool call: {:?}", tool_call);
                    }
                }
            }
            finish_reason => {
                println!("finish reason: {}", finish_reason);
                return;
            }
        }
    }
}
