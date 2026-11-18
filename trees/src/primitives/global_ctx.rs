// global prims

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use super::agent_ctx::{Agent, AgentKey};
use super::comms::{AgentRes, CommsErr, Req, Res};

pub struct Manager {
    virtual_path: Arc<Path>,
    snapshot_dir: PathBuf, // same btrfs fs as virtual_path
    sccache_dir: PathBuf,
    agents: HashMap<AgentKey, Agent>,
    listener: UnixListener,
}

impl Manager {
    pub fn new(
        virtual_path: PathBuf,
        snapshot_dir: PathBuf,
        sccache_dir: PathBuf,
        socket_path: &Path,
    ) -> io::Result<Self> {
        let _ = fs::remove_file(socket_path); // stale socket from a previous run
        let listener = UnixListener::bind(socket_path)?;

        Ok(Self {
            virtual_path: virtual_path.into(),
            snapshot_dir,
            sccache_dir,
            agents: HashMap::new(),
            listener,
        })
    }

    pub fn serve(&mut self) -> io::Result<()> {
        let listener = self.listener.try_clone()?;
        for stream in listener.incoming() {
            self.handle_conn(stream?)?;
        }
        Ok(())
    }

    fn handle_conn(&mut self, stream: UnixStream) -> io::Result<()> {
        let mut writer = stream.try_clone()?; // same socket, write half
        for line in BufReader::new(stream).lines() {
            let msg = line?;

            // parse message
            let res = match serde_json::from_str::<Req>(&msg) {
                Ok(Req::Agent(req)) => Res::Agent(
                    self.agents
                        .get(&req.key)
                        .map(|agent| AgentRes { agent: agent.clone() })
                        .ok_or(CommsErr::NotFound(req.key)),
                ),
                Ok(Req::Register(key)) => Res::Register(
                    self.register_agent(key)
                        .map_err(|e| CommsErr::Io(e.to_string())),
                ),
                Err(e) => Res::Invalid(CommsErr::BadRequest(e.to_string())),
            };

            // serialize agent response
            serde_json::to_writer(&mut writer, &res)?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn register_agent(&mut self, key: AgentKey) -> io::Result<PathBuf> {
        if self.agents.contains_key(&key) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("{key:?} already registered")));
        }

        let name = key.name();
        let host_path = self.snapshot_dir.join(&name);
        self.snapshot(&host_path)?;

        if let Err(e) = self.create_worktree(&name, &host_path) {
            let _ = delete_subvolume(&host_path);
            return Err(e);
        }

        let agent = Agent {
            host_path: host_path.clone(),
            virtual_path: Arc::clone(&self.virtual_path),
            sccache_dir: Arc::clone(&self.sccache_dir),
        };
        self.agents.insert(key, agent);
        Ok(host_path)
    }

    pub fn destroy_agent(&mut self, key: &AgentKey) -> io::Result<()> {
        let agent = self
            .agents
            .get(key)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("no agent {key:?}")))?;

        delete_subvolume(&agent.host_path)?;
        self.agents.remove(key);

        run(Command::new("git")
            .arg("-C")
            .arg(&*self.virtual_path)
            .args(["worktree", "prune"]))
    }

    fn create_worktree(&self, name: &str, mount_path: &Path) -> io::Result<()> {
        let branch = format!("trees/{name}");
        let tmp = self.snapshot_dir.join(format!(".tmp-{name}"));

        run(Command::new("git")
            .arg("-C")
            .arg(&*self.virtual_path)
            .args(["worktree", "add", "--no-checkout", "-b", &branch])
            .arg(&tmp)
            .arg("HEAD"))?;

        let snap_git = mount_path.join(".git");
        if snap_git.is_dir() {
            fs::remove_dir_all(&snap_git)?;
        } else {
            fs::remove_file(&snap_git)?;
        }
        fs::rename(tmp.join(".git"), &snap_git)?;
        fs::remove_dir(&tmp)?;

        run(Command::new("git")
            .arg("-C")
            .arg(&*self.virtual_path)
            .args(["worktree", "repair"])
            .arg(mount_path))?;

        run(Command::new("git")
            .arg("-C")
            .arg(mount_path)
            .args(["reset", "-q"]))
    }

    fn snapshot(&self, dest: &Path) -> io::Result<()> {
        run(Command::new("btrfs")
            .args(["subvolume", "snapshot"])
            .arg(&*self.virtual_path)
            .arg(dest))
    }
}

fn delete_subvolume(path: &Path) -> io::Result<()> {
    run(Command::new("btrfs").args(["subvolume", "delete"]).arg(path))
}

fn run(cmd: &mut Command) -> io::Result<()> {
    let out = cmd.output()?;
    if out.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{:?} failed: {}",
            cmd,
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}
