use std::io;
use std::path::Path;

pub fn unshare(flags: libc::c_int) -> io::Result<()> {
    todo!()
}

pub fn mount(source: Option<&Path>, target: &Path, flags: libc::c_ulong) -> io::Result<()> {
    todo!()
}

// exec command for agent over virtual fs
fn exec(&self, cmd: String) -> Result</* fill this in??? */> {
    // needs unsafe wrappers
}
