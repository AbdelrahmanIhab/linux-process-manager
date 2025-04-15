use libc::{kill, SIGKILL};

/// Sends SIGKILL to the specified PID.
/// Returns true if successful, false if an error occurred.
pub fn kill_process(pid: u32) -> bool {
    let pid = pid as i32; // libc expects i32
    let result = unsafe { kill(pid, SIGKILL) };
    result == 0
}
