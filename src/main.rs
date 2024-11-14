use askgql::process_inquiry;
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

    let gptcl = gptcl::GptClient::new(gptcl_hyper::HyperClient::new(), args.api_key.to_owned());
    let gql = askgql::gql::GqlClient::new(args.url, &args.authorization);

    process_inquiry(
        gql,
        &gptcl,
        &args.inquiry,
        &args.language,
        args.omit_schema_comments,
    )
    .await;
}
