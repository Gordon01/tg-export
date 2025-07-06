use std::path::PathBuf;

use clap::Parser;
use texport::Storage;

#[derive(Debug, Parser)]
struct Cli {
    /// A directory containing Telegram chat exports
    #[arg(long, short)]
    input: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let storage = cli
        .input
        .map(Storage::from_path)
        .unwrap_or_else(|| Storage::new())?;

    for (id, info) in storage.chats {
        println!("{id} → {info:?}");
    }

    Ok(())
}
