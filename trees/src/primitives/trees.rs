use std::ffi::CString;
use std::io;
use std::path::Path;
use libc;

pub fn unshare(flags: libc::c_int) -> io::Result<()> {
    unsafe {
        let res = libc::unshare(flags);
        match res {
            0 => Ok(()),
            _ => panic!("failed to unmount")
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
            _ => panic!("failed to unmount")
        }
    }
}

pub fn fork() -> io::Result<libc::pid_t> {
    unsafe {
        let res = libc::fork();
        match res {
            -1 => Err(io::Error::last_os_error()),
            pid => Ok(pid)
        }
    }
}

pub fn execvp(argv: &[*const libc::c_char]) -> io::Result<()> {
    unsafe {
        libc::execvp(argv[0], argv.as_ptr());
        Err(io::Error::last_os_error())
    }
}

pub fn exit(code: libc::c_int) -> ! {
    unsafe { libc::_exit(code) }
}

pub fn waitpid(pid: libc::pid_t) -> io::Result<libc::c_int> {
    let mut status: libc::c_int = 0;
    unsafe {
        let res = libc::waitpid(pid, &mut status, 0);
        match res {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(status)
        }
    }
}

pub fn agent_exec(source: &Path, target: &Path, cmd: &[String]) -> io::Result<i32> {
    if cmd.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty command"));
    }

    let args = cmd
        .iter()
        .map(|a| CString::new(a.as_bytes()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut argv: Vec<*const libc::c_char> = args.iter().map(|a| a.as_ptr()).collect();
    argv.push(std::ptr::null());

    let pid = fork()?;

    if pid == 0 {
        let setup = std::panic::catch_unwind(|| -> io::Result<()> {
            unshare(libc::CLONE_NEWNS)?;
            mount(None, Path::new("/"), libc::MS_REC | libc::MS_PRIVATE)?;
            mount(Some(source), target, libc::MS_BIND | libc::MS_REC)
        });
        if !matches!(setup, Ok(Ok(()))) {
            exit(126);
        }
        let _ = execvp(&argv);
        exit(127);
    }

    let status = loop {
        match waitpid(pid) {
            Ok(status) => break status,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    };

    if libc::WIFEXITED(status) {
        Ok(libc::WEXITSTATUS(status))
    } else {
        Ok(128 + libc::WTERMSIG(status))
    }
}
