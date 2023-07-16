#![cfg(target_family = "windows")]

// use crate::Color;
use crossterm::style::{ Stylize, Color };
use std::collections::HashSet;

use std::sync::Arc;
use regex::Regex;
use sysinfo::System;
use winreg::{ RegKey, enums::{ HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE } };

use crate::util::OSInfo;
pub struct WindowsInfo {}
impl OSInfo for WindowsInfo {
    fn displays(&self) -> Vec<String> {
        use std::{ sync::Mutex };

        use winsafe::{ HDC, prelude::{ Handle, user_Hdc } };
        let displays = Arc::new(Mutex::new(Vec::new()));
        let handle = HDC::NULL;
        handle
            .EnumDisplayMonitors(None, |_monitor, _hdc, rect| -> bool {
                displays
                    .lock()
                    .unwrap()
                    .push(format!("{}x{}", rect.right - rect.left, rect.bottom - rect.top));
                true
            })
            .ok();
        return displays.lock().unwrap().to_vec();
    }

    fn machine(&self) -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = hklm.open_subkey(r#"HARDWARE\DESCRIPTION\System\BIOS"#).ok()?;
        let res = format!(
            "{} ({})",
            subkey.get_value::<String, &str>("SystemProductName").ok().unwrap_or_default(),
            subkey.get_value::<String, &str>("SystemFamily").ok().unwrap_or_default()
            // subkey.get_value::<String, &str>("SystemVersion").ok().unwrap_or_default(),
            // subkey.get_value::<String, &str>("SystemSKU").ok().unwrap_or_default()
            // subkey.get_value::<String, &str>("SystemManufacturer").ok().unwrap_or_default()
        );
        Some(res)
    }

    fn theme(&self) -> Option<String> {
        let binding = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r#"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes"#)
            .ok()?
            .get_value::<String, &str>("CurrentTheme")
            .ok()?;
        let re = Regex::new(r#".*\\(.*)\."#).unwrap();
        let theme_name: &str = re.captures_iter(binding.as_str()).next()?.get(1)?.as_str();
        let dwm = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r#"Software\Microsoft\Windows\DWM"#)
            .ok()?;
        let color: u32 =
            (
                0x00ff_ffff &
                dwm
                    .get_value::<u32, &str>("AccentColor")
                    .or_else(|_| { dwm.get_value::<u32, &str>("ColorizationColor") })
                    .ok()?
            ).swap_bytes() >> 8;
        Some(
            format!("{theme_name} (#{color:X})")
                .on(Color::Rgb {
                    r: (color >> 16) as u8,
                    g: ((color >> 8) & 0xff) as u8,
                    b: (color & 0xff) as u8,
                })
                .to_string()
        )
    }

    fn kernel(&self, _s: &System) -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let current_version = hklm
            .open_subkey(r#"SOFTWARE\Microsoft\Windows NT\CurrentVersion"#)
            .ok()
            .unwrap();
        let major: u32 = current_version.get_value("CurrentMajorVersionNumber").ok()?;
        let minor: u32 = current_version.get_value("CurrentMinorVersionNumber").ok()?;
        let build_number: String = current_version.get_value("CurrentBuildNumber").ok()?;
        let version: String = current_version.get_value("DisplayVersion").ok()?;
        let ubr: u32 = current_version.get_value("UBR").ok()?;
        Some(format!("{major}.{minor}.{build_number}.{ubr} ({version})"))
    }

    fn wm(&self) -> Option<String> {
        if winsafe::DwmIsCompositionEnabled().ok()? {
            return Some("Desktop Window Manager".to_owned());
        }
        Some("Internal".to_owned())
    }

    fn de(&self) -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let current_version = hklm
            .open_subkey(r#"SOFTWARE\Microsoft\Windows NT\CurrentVersion"#)
            .ok()
            .unwrap();
        let major: u32 = current_version.get_value("CurrentMajorVersionNumber").ok()?;
        let minor: u32 = current_version.get_value("CurrentMinorVersionNumber").ok()?;
        if major >= 10 {
            return Some("Fluent".to_owned());
        } else if major >= 6 {
            if minor >= 2 {
                return Some("Metro".to_owned());
            }
            return Some("Aero".to_owned());
        }
        None
    }

    fn gpus(&self) -> Option<Vec<String>> {
        let video = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r#"SYSTEM\CurrentControlSet\Control\Video\"#)
            .ok()?;
        let res: Vec<String> = video
            .enum_keys()
            .filter_map(|x| {
                video
                    .open_subkey(x.ok().unwrap_or_default())
                    .map(|uuid| {
                        uuid.enum_keys()
                            .filter_map(
                                |y| -> Option<String> {
                                    uuid.open_subkey(y.unwrap())
                                        .ok()?
                                        .get_value::<String, &str>("DriverDesc")
                                        .ok()
                                }
                            )
                            .collect::<HashSet<String>>()
                    })
                    .ok()
            })
            .fold(HashSet::<String>::new(), |acc, x| { acc.union(&x).cloned().collect() })
            .into_iter()
            .collect();
        Some(res)
    }

    fn new() -> Self {
        Self {}
    }
}
