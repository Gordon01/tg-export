use std::{fs, path::PathBuf};

use clap::Parser;

use texport::{Chat, ChatStats, StatsSettings, Storage};

#[derive(Debug, Parser)]
struct Cli {
    /// A directory containing Telegram chat exports
    #[arg(long, short)]
    input: Option<PathBuf>,

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
    for input in Storage::new()?.chats.into_values().map(|v| v.path) {
        let json_data = fs::read(input)?;
        let chat: Chat = serde_json::from_slice(&json_data)?;
        stats.analyze(chat.messages);
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
