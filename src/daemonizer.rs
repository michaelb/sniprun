/**
Derived from https://github.com/immortal/fork
*/

/// Fork result
pub enum Fork {
    Parent(libc::pid_t),
    Child,
}

/// Close file descriptors stdin,stdout,stderr
///
/// # Errors
/// returns `-1` if error
pub fn close_fd() -> Result<(), i32> {
    match unsafe { libc::close(0) } {
        -1 => Err(-1),
        _ => match unsafe { libc::close(1) } {
            -1 => Err(-1),
            _ => match unsafe { libc::close(2) } {
                -1 => Err(-1),
                _ => Ok(()),
            },
        },
    }
}

pub fn fork() -> Result<Fork, i32> {
    let res = unsafe { libc::fork() };
    match res {
        -1 => Err(-1),
        0 => Ok(Fork::Child),
        res => Ok(Fork::Parent(res)),
    }
}

pub fn setsid() -> Result<libc::pid_t, i32> {
    let res = unsafe { libc::setsid() };
    match res {
        -1 => Err(-1),
        res => Ok(res),
    }
}
pub fn daemon() -> Result<Fork, i32> {
    match fork() {
        Ok(Fork::Parent(child_pid)) => Ok(Fork::Parent(child_pid)),
        Ok(Fork::Child) => setsid().and_then(|_| {
            close_fd()?;
            fork()
        }),
        Err(n) => Err(n),
    }
}
