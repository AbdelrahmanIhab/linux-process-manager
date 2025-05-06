use libc::{kill};

/// Sends a signal (SIGKILL, SIGSTOP, SIGCONT) to a process.
/// Returns true if successful, false otherwise.
pub fn send_signal(pid: u32, signal: i32) -> bool {
    let result = unsafe { kill(pid as i32, signal) };
    result == 0
}
