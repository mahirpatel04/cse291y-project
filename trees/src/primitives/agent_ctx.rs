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
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentKey {
    pub session_id: String,
    pub agent_id: Option<String>,
}

impl AgentKey {
    pub fn name(&self) -> String {
        format!("agent-{}-{}", self.session_id, self.agent_id.as_deref().unwrap_or("main"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
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
