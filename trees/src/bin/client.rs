use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use trees::primitives::agent_ctx::AgentKey;
use trees::primitives::comms::{AgentReq, CommsErr, Req, Res};

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

// exec command for agent over virtual fs
fn exec(socket_path: &Path, key: AgentKey, cmd: &[String]) -> Result<i32, CommsErr> {
    let io_err = |e: io::Error| CommsErr::Io(e.to_string());

    let mut stream = UnixStream::connect(socket_path).map_err(io_err)?;
    let mut msg = serde_json::to_string(&Req::Agent(AgentReq { key })).map_err(|e| CommsErr::BadRequest(e.to_string()))?;
    msg.push('\n');
    stream.write_all(msg.as_bytes()).map_err(io_err)?;

    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).map_err(io_err)?;

    let agent = match serde_json::from_str::<Res>(&line) {
        Ok(Res::Agent(res)) => res?.agent,
        Ok(Res::Invalid(e)) => return Err(e),
        Ok(other) => return Err(CommsErr::BadRequest(format!("unexpected response: {other:?}"))),
        Err(e) => return Err(CommsErr::BadRequest(e.to_string())),
    };

    agent.exec(cmd).map_err(io_err)
}

// register command for agent
fn register(socket_path: &Path, key: AgentKey) -> Result<PathBuf, CommsErr> {
    let io_err = |e: io::Error| CommsErr::Io(e.to_string());

    let mut stream = UnixStream::connect(socket_path).map_err(io_err)?;
    let mut msg = serde_json::to_string(&Req::Register(key)).map_err(|e| CommsErr::BadRequest(e.to_string()))?;
    msg.push('\n');
    stream.write_all(msg.as_bytes()).map_err(io_err)?;

    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).map_err(io_err)?;

    match serde_json::from_str::<Res>(&line) {
        Ok(Res::Register(res)) => res,
        Ok(Res::Invalid(e)) => Err(e),
        Ok(other) => Err(CommsErr::BadRequest(format!("unexpected response: {other:?}"))),
        Err(e) => Err(CommsErr::BadRequest(e.to_string())),
    }
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
        Cmd::Exec { session_id, agent_id, cmd } => {
            let key = AgentKey { session_id, agent_id: Some(agent_id) };
            match exec(&cli.socket, key, &cmd) {
                Ok(code) => ExitCode::from(code as u8),
                Err(e) => {
                    eprintln!("{e:?}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
