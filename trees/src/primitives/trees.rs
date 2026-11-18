use std::ffi::CString;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::ptr;

use super::agent_ctx::AgentKey;
use super::comms::{CommsErr, Req, Res};

// two unix wrappers
pub fn unshare(flags: libc::c_int) -> io::Result<()> {
    match unsafe { libc::unshare(flags) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}



pub fn mount(source: Option<&Path>, target: &Path, flags: libc::c_ulong) -> io::Result<()> {
    let src = source.map(|s| CString::new(s.as_os_str().as_bytes())).transpose()?;
    let dest = CString::new(target.as_os_str().as_bytes())?;
    let src_ptr = src.as_ref().map_or(ptr::null(), |s| s.as_ptr());

    match unsafe { libc::mount(src_ptr, dest.as_ptr(), ptr::null(), flags, ptr::null()) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

// exec command for agent over virtual fs
fn exec(agent: i32, cmd: String) {

}

// register command for agent
pub fn register(socket_path: &Path, key: AgentKey) -> Result<PathBuf, CommsErr> {
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