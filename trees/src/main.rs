use std::io;
use std::path::PathBuf;

use clap::Parser;
use trees::primitives::global_ctx::Manager;

#[derive(Parser)]
struct Cli {
    socket: PathBuf,
    virtual_path: PathBuf,
    snapshot_dir: PathBuf,
    sccache_dir: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut manager = Manager::new(cli.virtual_path, cli.snapshot_dir, cli.sccache_dir, &cli.socket)?;
    manager.serve()
}
