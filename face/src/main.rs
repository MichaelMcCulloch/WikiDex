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

pub(crate) struct Config {
    pub(crate) index_path: PathBuf,
    pub(crate) host: String,
    pub(crate) port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();
    let index_path = args.index_path;

    let faiss_index = index::FaissIndex::new(&index_path)?;

    let system_runner = rt::System::new();

    let exec = async {
        let index_engine = IndexEngine::new(faiss_index).await;
        let _ = run_server(index_engine, args.host, args.port)
            .unwrap()
            .await;
    };

    system_runner.block_on(exec);
    Ok(())
}
