use nix::{
    libc::user_regs_struct,
    sys::{
        ptrace,
        signal::Signal::SIGTRAP,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::ffi::c_void;
use syscalls::Sysno;
use sysinfo::{ProcessExt, System, SystemExt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("Process not found `{0}`")]
    ProcessNotFound(String),
    #[error("Nix error `{0}`")]
    NixError(#[from] nix::errno::Errno),
    #[error("Unexpected Wait Status `{0:#?}`")]
    UnexpectedWaitStatus(WaitStatus),
    #[error("StdIo Error `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Mmap Error `{0:#?}`")]
    MmapBadAddress(u64),
    #[error("Munmap Error `{0:#?}`")]
    MunmapFailed(u64),
}
pub struct UProc {
    pid: Pid,
}

impl UProc {
    pub fn attach(pid: Pid) -> Result<Self, HostError> {
        ptrace::attach(pid)?;

        log::info!("victim pid: {}", pid);
        Ok(Self { pid })
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    fn mem_path(&self) -> String {
        format!("/proc/{}/mem", self.pid.as_raw() as u32)
    }

    pub fn mem_read(&self, addr: u64, len: usize) -> Result<Vec<u8>, HostError> {
        use std::os::unix::fs::FileExt;

        let mut data = vec![0u8; len];
        let mem = std::fs::File::open(self.mem_path())?;
        let read = mem.read_at(&mut data, addr)?;

        data.truncate(read);
        Ok(data)
    }

    pub fn mem_write(&self, addr: u64, data: &[u8]) -> Result<usize, HostError> {
        use std::os::unix::fs::FileExt;
        let mem = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.mem_path())?;
        let len = mem.write_at(data, addr)?;
        Ok(len)
    }

    fn wait(&self) -> Result<WaitStatus, HostError> {
        waitpid(self.pid, None).map_err(HostError::NixError)
    }

    fn sstep(&self) -> Result<(), HostError> {
        ptrace::step(self.pid, None)?;
        match self.wait()? {
            WaitStatus::Stopped(_, SIGTRAP) => Ok(()),
            status => Err(HostError::UnexpectedWaitStatus(status)),
        }
    }

    pub fn syscall(
        &self,
        syscall: Sysno,
        rdi: u64,
        rsi: u64,
        rdx: u64,
        r10: u64,
        r8: u64,
        r9: u64,
    ) -> Result<user_regs_struct, HostError> {
        log::trace!("pid: {} syscall: {:#?}", self.pid, syscall);
        let syscall_inst = [0x0Fu8, 0x05u8];

        let regs = ptrace::getregs(self.pid)?;
        let ip = regs.rip;
        let inst = self.mem_read(ip, syscall_inst.len())?;
        self.mem_write(ip, &syscall_inst)?;

        let new_regs = {
            let mut new_regs = regs;
            new_regs.rax = syscall as u64;
            new_regs.rdi = rdi;
            new_regs.rsi = rsi;
            new_regs.rdx = rdx;
            new_regs.r10 = r10;
            new_regs.r8 = r8;
            new_regs.r9 = r9;
            new_regs
        };
        ptrace::setregs(self.pid, new_regs)?;

        self.sstep()?;
        let result = ptrace::getregs(self.pid)?;

        self.mem_write(ip, &inst)?;
        ptrace::setregs(self.pid, regs)?;

        Ok(result)
    }

    pub fn malloc(
        &self,
        addr: u64,
        len: u64,
        prot: u64,
        flags: u64,
        fd: u64,
        offset: u64,
    ) -> Result<UProcMem, HostError> {
        let mmap = self.syscall(Sysno::mmap, addr, len, prot, flags, fd, offset)?;
        let mmap = mmap.rax;

        if mmap == 0 {
            return Err(HostError::MmapBadAddress(addr));
        }

        Ok(UProcMem {
            addr: mmap,
            owner: self,
            len,
        })
    }

    fn free(&self, addr: u64, len: u64) -> Result<(), HostError> {
        let munmap_result = self.syscall(Sysno::munmap, addr, len, 0, 0, 0, 0)?;
        let munmap_result = munmap_result.rax;

        if munmap_result != 0 {
            return Err(HostError::MunmapFailed(addr));
        }

        Ok(())
    }
}

impl Drop for UProc {
    fn drop(&mut self) {
        if let Err(e) = ptrace::detach(self.pid, None) {
            log::error!("failed to detach from pid: {} with err: {:#?}", self.pid, e);
        } else {
            log::info!("detach from pid: {}", self.pid);
        }
    }
}

pub struct UProcMem<'a> {
    owner: &'a UProc,
    pub addr: u64,
    pub len: u64,
}
