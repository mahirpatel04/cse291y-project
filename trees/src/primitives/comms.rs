use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use super::agent_ctx::{Agent, AgentKey};

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentReq {
    pub key: AgentKey
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Req {
    Agent(AgentReq),
    Register(AgentKey)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentRes {
    pub agent: Agent
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommsErr {
    NotFound(AgentKey),
    BadRequest(String),
    Io(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Res {
    Agent(Result<AgentRes, CommsErr>),
    Register(Result<PathBuf, CommsErr>),
    Invalid(CommsErr),
}
