use std::io;
use std::path::Path;
use libc;

pub fn unshare(flags: libc::c_int) -> io::Result<()> {
    // let flags : i32 = libc::CLONE_NEWNS;

    unsafe { 
        let res = libc::unshare(flags);
        match res {
            0 => Ok(()),
            _ => panic!("failed to unmount") // panicking for now? think that is most reasonable across the ffi boundary
        }
    }
}



pub fn mount(source: Option<&Path>, target: &Path, flags: libc::c_ulong) -> io::Result<()> {
    let src: &str= match source {
        Some(s) => s.to_str().unwrap_or("none"),
        _ => "none"
    };

    let dest= target.to_str().ok_or("oops")?;

    let fs_type = "ext4";

    let data = std::ptr::null();

    unsafe {
        let res = libc::mount(src.as_ptr(), dest.as_ptr(), fs_type.as_ptr(), flags, data);
        match res {
            0 => Ok(()),
            _ => panic!("failed to unmount") // panicking for now? think that is most reasonable across the ffi boundary
        }
    }
}

// exec command for agent over virtual fs
fn exec(&self, cmd: String) -> Result</* fill this in??? */> {
    // needs unsafe wrappers
}
