// global prims

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use super::agent_ctx::Agent;

pub struct Manager {
    virtual_path: Arc<Path>,
    snapshot_dir: PathBuf, // same btrfs fs as virtual_path
    sccache_dir: PathBuf,
    next_id: u64,
    agents: HashMap<u64, Agent>,
}

// TODO: serde over unix socket

impl Manager {
    pub fn new(virtual_path: PathBuf, snapshot_dir: PathBuf, sccache_dir: PathBuf) -> Self {
        Self {
            virtual_path: virtual_path.into(),
            snapshot_dir,
            sccache_dir,
            next_id: 0,
            agents: HashMap::new(),
        }
    }

    pub fn sccache_dir(&self) -> &Path {
        &self.sccache_dir
    }

    pub fn agent(&self, id: u64) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn agent_for_path(&self, path: &Path) -> Option<&Agent> {
        self.agents.values().find(|a| path.starts_with(&a.host_path))
    }

    pub fn register_agent(&mut self) -> io::Result<&Agent> {
        let id = self.next_id;
        self.next_id += 1;

        let host_path = self.snapshot_dir.join(format!("agent-{id}"));
        self.snapshot(&host_path)?;

        if let Err(e) = self.create_worktree(id, &host_path) {
            let _ = delete_subvolume(&host_path);
            return Err(e);
        }

        let agent = Agent {
            id,
            host_path,
            virtual_path: Arc::clone(&self.virtual_path),
        };
        self.agents.insert(id, agent);
        Ok(&self.agents[&id])
    }

    pub fn destroy_agent(&mut self, id: u64) -> io::Result<()> {
        let agent = self
            .agents
            .get(&id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("no agent {id}")))?;

        delete_subvolume(&agent.host_path)?;
        self.agents.remove(&id);

        run(Command::new("git")
            .arg("-C")
            .arg(&*self.virtual_path)
            .args(["worktree", "prune"]))
    }

    fn create_worktree(&self, id: u64, mount_path: &Path) -> io::Result<()> {
        let branch = format!("trees/agent-{id}");
        let tmp = self.snapshot_dir.join(format!(".tmp-agent-{id}"));

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
