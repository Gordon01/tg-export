use std::{fs, path::PathBuf};

use clap::Parser;

use tg_export::{Chat, ChatStats, StatsSettings};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, short)]
    input: Vec<PathBuf>,

    #[arg(long, short, default_value = "text")]
    output: OutputFormat,

    #[arg(long, short, default_value_t = 10)]
    max_words: usize,

    #[arg(long, short, default_value_t = 5)]
    participants: usize,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut stats = ChatStats {
        settings: StatsSettings {
            max_words: cli.max_words,
            max_participants: cli.participants,
            ..Default::default()
        },
        ..Default::default()
    };
    for input in cli.input {
        let json_data = fs::read(input)?;
        let chat: Chat = serde_json::from_slice(&json_data)?;
        stats.analyze(&chat.messages);
    }

    println!(
        "{}",
        match cli.output {
            OutputFormat::Text => stats.to_string(),
            OutputFormat::Json => serde_json::to_string_pretty(&stats)?,
        }
    );

    Ok(())
}
