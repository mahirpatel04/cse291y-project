use std::io;
use std::path::Path;
use libc;

pub fn unshare(flags: libc::c_int) -> io::Result<()> {
    let flags : i32 = libc::CLONE_NEWNS;

    match unsafe { libc::unshare(flags) } {
        c
    }
}

pub fn mount(source: Option<&Path>, target: &Path, flags: libc::c_ulong) -> io::Result<()> {
    todo!()
}

// exec command for agent over virtual fs
fn exec(&self, cmd: String) -> Result</* fill this in??? */> {
    // needs unsafe wrappers
}
