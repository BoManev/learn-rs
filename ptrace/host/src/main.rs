use byteorder::{LittleEndian, ReadBytesExt};
use host::{HostError, UProcMem, UProc};
use nix::{
    libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE},
    unistd::Pid,
};
use syscalls::Sysno;
use sysinfo::{ProcessExt, System, SystemExt};

fn main() -> Result<(), HostError> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let process_name = "victim";
    log::info!("host pid: {}", std::process::id());

    let mut sys = System::new_all();
    sys.refresh_all();

    let process = sys
        .processes_by_name(process_name)
        .take(1)
        .next()
        .ok_or_else(|| HostError::ProcessNotFound(process_name.to_string()))?;

    let pid = Pid::from_raw(process.pid().into());

    let proc = UProc::attach(pid)?;
    let umem = proc.malloc(
        0,
        8,
        (PROT_READ | PROT_WRITE) as u64,
        (MAP_PRIVATE | MAP_ANONYMOUS) as u64,
        u64::MAX,
        0,
    )?;
    log::info!("mem addr: {:#X}", umem.addr);

    let read = proc.mem_read(umem.addr, umem.len as usize);
    log::info!("malloc: {:?}", read);

    let _res = proc.syscall(Sysno::time, umem.addr, 0, 0, 0, 0, 0)?;
    let read = proc.mem_read(umem.addr, umem.len as usize)?;

    let mut read = std::io::Cursor::new(read);
    let output =  read.read_i64::<LittleEndian>()?;

    log::info!("Time Output: {:?}", output);
    Ok(())
}
