# AskGQL

AskGQL is a conversational LLM assistant that can interact with GraphQL servers. Users can ask questions in a conversational manner, and AskGQL can generate GraphQL queries and send them to the server.

![demo](demo.png)

## Usage

You need to have an OpenAI API key to use this tool. You can get one [here](https://platform.openai.com/signup).

### Installation

```bash
git clone https://github.com/carrotflakes/askgql.git
cd askgql
cargo install --path .
```

### Run

```bash
askgql -u "https://countries.trevorblades.com/" -a $OPENAI_API_KEY
```

### Run without installation

```bash
git clone https://github.com/carrotflakes/askgql.git
cd askgql
cargo run -- -u "https://countries.trevorblades.com/" -a $OPENAI_API_KEY
```

## License

MIT
