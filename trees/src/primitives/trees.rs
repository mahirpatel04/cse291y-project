use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use libc;

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
        let setup = unshare(libc::CLONE_NEWNS)
            .and_then(|_| mount(None, Path::new("/"), libc::MS_REC | libc::MS_PRIVATE))
            .and_then(|_| mount(Some(source), target, libc::MS_BIND | libc::MS_REC));

        match setup {
            Ok(()) => {
                let _ = execvp(&argv);
                exit(127)
            }
            Err(_) => exit(126),
        }
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

