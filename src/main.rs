use askgql::process_interactive;
use clap::Parser;

/// A simple CLI to interact with a GraphQL server.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL of the GraphQL server.
    #[arg(short, long)]
    url: String,

    /// The inquiry to start the conversation.
    #[arg(short, long)]
    inquiry: Option<String>,

    /// OpenAI API key.
    #[arg(short, long)]
    api_key: String,

    /// The authorization token to send to the GraphQL server.
    #[arg(long)]
    authorization: Option<String>,

    /// Omit the comments from the GraphQL schema.
    #[arg(long)]
    omit_schema_comments: bool,

    /// Schema file.
    #[arg(short, long)]
    schema: Option<String>,

    /// Print schema and exit.
    #[arg(long)]
    print_schema: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("api server url: {}", args.url);

    let gptcl = gptcl::GptClient::new(gptcl_hyper::HyperClient::new(), args.api_key.to_owned());
    let gql = askgql::gql::GqlClient::new(args.url, &args.authorization);

    let schema = if let Some(schema) = &args.schema {
        std::fs::read_to_string(schema).unwrap()
    } else {
        gql.introspect(args.omit_schema_comments).await.unwrap()
    };
    if args.print_schema {
        println!("schema:\n{}", schema);
        return;
    }

    process_interactive(gql, &gptcl, schema, args.inquiry.clone()).await;
}
