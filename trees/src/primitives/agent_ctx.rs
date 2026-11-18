// define the subagent prims

/*
    global ctx -> initial agent state
    when you launch a subagent
    create a new Agent
    1. create a new dir (cow, via btrfs)
    2. track the directory and parent
    3.
*/

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use super::trees;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    pub host_path: PathBuf,
    pub virtual_path: Arc<Path>,
    pub sccache_dir: Arc<Path>,
}

impl Agent {
    /// Run `cmd` (program + args) with this agent's snapshot mounted over
    /// `virtual_path`. Returns the exit code.
    pub fn exec(&self, cmd: &[String]) -> io::Result<i32> {
        // TODO: apply sccache setup (env vars from self.sccache_dir) to the command
        trees::agent_exec(&self.host_path, &self.virtual_path, cmd)
    }

    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(&*self.virtual_path) {
            Ok(rest) if !path.starts_with(&self.host_path) => self.host_path.join(rest),
            _ => path.to_path_buf(),
        }
    }
}
