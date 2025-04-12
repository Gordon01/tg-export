use std::{fs, path::PathBuf};

use clap::Parser;

use tg_export::{Chat, ChatStats};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, short)]
    input: PathBuf,

    #[arg(long, short, default_value = "text")]
    output: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let json_data = fs::read(cli.input)?;
    let export: Chat = serde_json::from_slice(&json_data)?;
    let stats = ChatStats::analyze(&export.messages);

    println!(
        "{}",
        match cli.output {
            OutputFormat::Text => stats.to_string(),
            OutputFormat::Json => serde_json::to_string_pretty(&stats)?,
        }
    );

    Ok(())
}
