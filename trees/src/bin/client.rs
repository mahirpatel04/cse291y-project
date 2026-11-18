use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use trees::primitives::agent_ctx::AgentKey;
use trees::primitives::trees::register;

#[derive(Parser)]
struct Cli {
    socket: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Register { session_id: String, agent_id: String },
    Exec { session_id: String, agent_id: String, cmd: Vec<String> },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Register { session_id, agent_id } => {
            let key = AgentKey { session_id, agent_id: Some(agent_id) };
            match register(&cli.socket, key) {
                Ok(path) => {
                    println!("{}", path.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e:?}");
                    ExitCode::FAILURE
                }
            }
        }
        Cmd::Exec { .. } => todo!("exec"),
    }
}
