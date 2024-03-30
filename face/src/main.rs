use std::path::PathBuf;

use actix_web::rt;
use clap::Parser;
use engine::IndexEngine;
use server::run_server;

mod api;
mod engine;
mod index;
mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    index_path: PathBuf,
    #[arg( long, default_value_t = String::from("0.0.0.0"))]
    pub(crate) host: String,
    #[arg(long, default_value_t = 6947)]
    pub(crate) port: u16,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let index_path = args.index_path;

    let faiss_index = index::FaissIndex::new(&index_path)?;

    let index_engine = IndexEngine::new(faiss_index);

    let server = run_server(index_engine, args.host, args.port)?;

    let system_runner = rt::System::new();
    system_runner.block_on(server).map_err(anyhow::Error::from)
}
