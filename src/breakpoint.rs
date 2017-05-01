
use std::ptr;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::*;
use nix::libc::{pid_t, c_void, c_long};
use nix::Result;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const INT: u8 = 0xCC;

const RIP: u8 = 128;

/// Breakpoint construct used to monitor program execution. As tarpaulin is an
/// automated process, this will likely have less functionality than most 
/// breakpoint implementations.
#[derive(Debug)]
pub struct Breakpoint { 
    /// Current process id
    pub pid: pid_t,
    /// Program counter
    pub pc: u64,
    /// Bottom byte of address data. 
    /// This is replaced to enable the interrupt. Rest of data is never changed.
    data: i64,
}

impl Breakpoint {
    
    pub fn new(pid:pid_t, pc:u64) ->Result<Breakpoint> {
        
        let data = ptrace(PTRACE_PEEKDATA, pid, pc as * mut c_void, ptr::null_mut())?;
        let mut b = Breakpoint{ 
            pid: pid,
            pc: pc,
            data: data,
        };
        match b.enable() {
            Ok(_) => Ok(b),
            Err(e) => Err(e)
        }
    }

    /// Attaches the current breakpoint.
    fn enable(&mut self) -> Result<c_long> {
        let intdata = (self.data & !(0xFF as i64)) | (INT as i64);
        let intdata = intdata as * mut c_void;
        ptrace(PTRACE_POKEDATA, self.pid, self.pc as * mut c_void, intdata) 
    }
    
    fn disable(&self) -> Result<c_long> {
        let raw_addr = self.pc as * mut c_void;
        ptrace(PTRACE_POKEDATA, self.pid, raw_addr, self.data as * mut c_void) 
    }

    /// Steps past the current breakpoint.
    /// For more advanced coverage may interrogate the variables of a branch.
    pub fn step(&mut self) -> Result<c_long> {
        self.disable()?;
        // Need to set the program counter back one. 
        ptrace(PTRACE_POKEUSER, self.pid, RIP as * mut c_void, self.pc as * mut c_void)?;
        ptrace(PTRACE_SINGLESTEP, self.pid, ptr::null_mut(), ptr::null_mut())
        //self.enable()
    }
}
