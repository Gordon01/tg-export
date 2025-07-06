# Telegram Chat Analyzer

A tool for analyzing [Telegram chat exports](https://telegram.org/blog/export-and-more) in JSON format.

# Usage

To run the example analyzer:

```bash
cargo r -p texport --example full
```

This command will process all available chat exports and display combined statistics of your messages in a readable format.

## Input Requirements
Place your exported Telegram chats (in JSON format) into the default directory. Each chat export must include a result.json file inside its folder.

## Output Formats

You can choose the output format using the `--output` flag:

* Default: Human-readable text (printed to stdout)
* JSON: Machine-readable format
