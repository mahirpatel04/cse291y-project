// define the subagent prims

/*
    global ctx -> initial agent state
    when you launch a subagent
    create a new Agent
    1. create a new dir (cow, via btrfs)
    2. track the directory and parent
    3.
*/

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Agent {
    pub id: u64,
    pub host_path: PathBuf,
    pub virtual_path: Arc<Path>,
}

impl Agent {
    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(&*self.virtual_path) {
            Ok(rest) if !path.starts_with(&self.host_path) => self.host_path.join(rest),
            _ => path.to_path_buf(),
        }
    }
}
