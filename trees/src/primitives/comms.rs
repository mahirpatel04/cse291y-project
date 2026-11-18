use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use super::agent_ctx::Agent;

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentReq {
    pub key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Req {
    Agent(AgentReq),
    Register(String),
    Destroy(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentRes {
    pub agent: Agent
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CommsErr {
    NotFound(String),
    BadRequest(String),
    Io(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Res {
    Agent(Result<AgentRes, CommsErr>),
    Register(Result<PathBuf, CommsErr>),
    Destroy(Result<(), CommsErr>),
    Invalid(CommsErr),
}
