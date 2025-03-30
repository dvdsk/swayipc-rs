use std::env;
use std::ffi::OsStr;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use swayipc_types::{Error, Fallible};
use sysinfo::System;
use uzers::get_current_uid;

fn calc_socketpath() -> Fallible<String> {
    let user_id = get_current_uid();
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let sway_pid = sys
        .processes_by_exact_name(OsStr::new("sway"))
        .next()
        .map(|p| p.pid())
        .ok_or(Error::CouldNotFindSwayProcess)?;

    Ok(format!(
        "/run/user/{user_id}/sway-ipc.{user_id}.{sway_pid}.sock"
    ))
}

pub fn get_socketpath() -> Fallible<PathBuf> {
    env::var("I3SOCK")
        .or_else(|_| env::var("SWAYSOCK"))
        .or_else(|_| spawn("i3"))
        .or_else(|_| spawn("sway"))
        .or_else(|_| calc_socketpath())
        .map_err(|_| Error::SocketNotFound)
        .map(PathBuf::from)
}

fn spawn(wm: &str) -> Fallible<String> {
    let mut child = Command::new(wm)
        .arg("--get-socketpath")
        .stdout(Stdio::piped())
        .spawn()?;
    let mut buf = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_string(&mut buf)?;
        buf.pop();
    }
    child.wait()?;
    Ok(buf)
}
