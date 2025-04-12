use std::{fs, path::PathBuf};

use clap::Parser;

use tg_export::Chat;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, short)]
    input: PathBuf,

    #[arg(long, short)]
    max: Option<usize>,

    #[arg(long, short)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let json_data = fs::read(cli.input)?;
    let export: Chat = serde_json::from_slice(&json_data)?;

    if let Some(out) = cli.output {
        let mut file = fs::File::create(out)?;
        export.write_export(&mut file, cli.max)?;
    } else {
        export.write_export(&mut std::io::stdout(), cli.max)?;
    }

    Ok(())
}
