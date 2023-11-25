#![cfg(test)]
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Command;

use crate::info::Info;
lazy_static! {
    static ref FF_INFO: HashMap<String, String> = {
        let mut ff_tmp = HashMap::new();
        let output: String = String::from_utf8_lossy(
            Command::new("fastfetch")
                .args(["--pipe"])
                .output()
                .expect("Failed to execute fastfetch")
                .stdout
                .as_slice(),
        )
        .to_string();
        let mut lines = output.lines();
        let (ff_username, ff_hostname) = lines
            .next()
            .expect("Could not parse username from fastfetch output")
            .split_once('@')
            .expect("Could not parse hostname from fastfetch output");
        ff_tmp.insert("username".to_string(), ff_username.into());
        ff_tmp.insert("hostname".to_string(), ff_hostname.into());
        lines.next();
        for line in lines {
            if let Some((x, y)) = line.split_once(": ") {
                ff_tmp.insert(x.into(), y.into());
            }
        }
        ff_tmp
    };
    static ref MIRA_INFO: Info = Info::new();
}
#[test]
fn test_username() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("username").unwrap()).trim_matches('\0'),
        &(MIRA_INFO.username.as_ref().unwrap().as_str())
    );
}
#[test]
fn test_hostname() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("hostname").unwrap()).trim(),
        &(MIRA_INFO.hostname.as_ref().unwrap().as_str())
    );
}
#[test]
#[ignore = "Mirafetch returns this differently from fastfetch"]
fn test_os() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("OS").unwrap()).trim(),
        &(MIRA_INFO.os.as_ref().unwrap())
    );
}
#[test]
fn test_kernel() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("Kernel").unwrap()).trim(),
        &(MIRA_INFO.kernel.as_ref().unwrap())
    );
}
#[test]
#[ignore = "Mirafetch returns this differently from fastfetch"]
fn test_cpu() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("CPU").unwrap()).trim(),
        &(MIRA_INFO.cpu.as_ref().unwrap())
    );
}

#[test]
fn test_host() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("Host").unwrap()).trim(),
        &(MIRA_INFO.machine.as_ref().unwrap())
    );
}
#[test]
fn test_uptime() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("Uptime").unwrap()).trim(),
        &(MIRA_INFO.uptime.as_ref().unwrap())
    );
}
#[test]
fn test_de() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("DE").unwrap()).trim(),
        &(MIRA_INFO.de.as_ref().unwrap())
    );
}
#[test]
fn test_wm() {
    assert_eq!(
        &AsRef::<str>::as_ref(FF_INFO.get("WM").unwrap()).trim(),
        &(MIRA_INFO.wm.as_ref().unwrap())
    );
}
