# AskGQL

AskGQL is a natural language interface for GraphQL servers.

![demo](demo.png)

## Usage

You need to have an OpenAI API key to use this tool. You can get one [here](https://platform.openai.com/signup).

Make sure that the GraphQL server supports introspection.

### Installation

```bash
cargo install --path .
```

### Run

```bash
askgql -u "https://countries.trevorblades.com/" -i "What is the official language of India?" -a $OPENAI_API_KEY
```

If you want to specify the language of the input question, you can use the `-l` option.

```bash
askgql -u "https://countries.trevorblades.com/" -i "インドの公用語" -l ja -a $OPENAI_API_KEY
```

## License

MIT
