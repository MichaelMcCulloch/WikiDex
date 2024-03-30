use std::path::PathBuf;

use anyhow::Ok;
use clap::Parser;

mod index;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    index_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let index_path = args.index_path;
    let _faiss_index = index::FaissIndex::new(&index_path)?;

    Ok(())
}
