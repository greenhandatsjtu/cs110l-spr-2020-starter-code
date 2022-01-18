use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::{Child, Command};
use std::os::unix::process::CommandExt;
use std::mem::size_of;
use nix::sys::signal::Signal;
use crate::dwarf_data::{DwarfData, Line};

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
pub fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &Vec<usize>) -> Option<Inferior> {
        let mut cmd = Command::new(&target);
        cmd.args(args);
        unsafe {
            cmd.pre_exec(child_traceme);
        }
        let mut inferior = Inferior { child: cmd.spawn().ok()? };
        let status = inferior.wait(Some(WaitPidFlag::WUNTRACED)).ok()?;
        if let Status::Stopped(signal, _) = status {
            if signal != Signal::SIGTRAP {
                return None;
            }
            for bp in breakpoints {
                let res = inferior.write_byte(*bp, 0xcc);
                if res.is_err() {
                    println!("Unable to set breakpoint at {:#x}", bp);
                }
            }
            return Some(inferior);
        }
        None
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }
    // make process to continue executing
    pub fn cont(&self) -> Result<Status, nix::Error> {
        ptrace::cont(self.pid(), None)?;
        self.wait(Some(WaitPidFlag::WUNTRACED))
    }
    // kill inferior process
    pub fn kill(&mut self) -> Result<Status, nix::Error> {
        println!("Killing running inferior (pid {})", self.pid());
        self.child.kill().unwrap();
        self.wait(None)
    }

    //
    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid())?;
        let mut rip = regs.rip;
        let mut rbp = regs.rbp;
        loop {
            let line = match debug_data.get_line_from_addr(rip as usize) {
                None => {
                    println!("There's no code at {:#x}", rip);
                    return Ok(());
                }
                Some(val) => val
            };
            let func = match debug_data.get_function_from_addr(rip as usize) {
                None => return Ok(()),
                Some(val) => val
            };
            println!("{} ({})", func, line);
            if func == "main" {
                break;
            }
            rip = ptrace::read(self.pid(), (rbp + 8) as ptrace::AddressType)? as u64;
            rbp = ptrace::read(self.pid(), rbp as ptrace::AddressType)? as u64;
        }
        Ok(())
    }

    // write byte val to given address and return original byte
    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }
}
