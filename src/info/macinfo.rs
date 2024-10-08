#![cfg(target_os = "macos")]
use super::OSInfo;
use arcstr::ArcStr;
use libc::timespec;
use platform_info::{PlatformInfo, PlatformInfoAPI, UNameAPI};
use core::ffi::CStr;
use rustc_hash::FxHashMap;
use std::{alloc::Layout, env};
pub struct MacInfo {
    uts: PlatformInfo
}

impl MacInfo {
    pub fn new() -> Self {
        Self {
            uts: PlatformInfo::new().unwrap()
        }
    }
}

impl OSInfo for MacInfo {
    fn os(&self) -> Option<ArcStr> {
        let info = plist::from_file::<&str, FxHashMap<String, String>>(
            "/System/Library/CoreServices/SystemVersion.plist",
        )
        .ok()?;

        let name = info.get("ProductName").unwrap().as_str();
        let version: &str = info.get("ProductUserVisibleVersion").unwrap().as_str();
        let mut split = version.split(".");
        let codename: &str = match split.next() {
            Some("15") => "Sequoia",
            Some("14") => "Sonoma",
            Some("13") => "Ventura",
            Some("12") => "Monterey",
            Some("11") => "Big Sur",
            Some("10") => match split.next() {
                Some("16") => "Big Sur",
                Some("15") => "Catalina",
                Some("14") => "Mojave",
                Some("13") => "High Sierra",
                Some("12") => "Sierra",
                Some("11") => "El Capitan",
                Some("10") => "Yosemite",
                Some("9") => "Mavericks",
                Some("8") => "Mountain Lion",
                Some("7") => "Lion",
                Some("6") => "Snow Leopard",
                Some("5") => "Leopard",
                Some("4") => "Tiger",
                Some("3") => "Panther",
                Some("2") => "Jaguar",
                Some("1") => "Puma",
                Some("0") => "Cheetah",
                _ => panic!("Unknown Mac OS X Version"),
            },
            _ => panic!("Unknown macOS Version"),
        };
        return Some(arcstr::format!("{name} {codename} {version}"));
    }

    fn sys_font(&self) -> Option<ArcStr> {
        None
    }

    fn cursor(&self) -> Option<ArcStr> {
        None
    }

    fn terminal(&self) -> Option<ArcStr> {
        None
    }

    fn term_font(&self) -> Option<ArcStr> {
        None
    }

    fn gpus(&self) -> Vec<ArcStr> {
        Vec::new()
    }

    fn memory(&self) -> Option<ArcStr> {
        None
    }

    fn disks(&self) -> Vec<(ArcStr, ArcStr)> {
        Vec::new()
    }

    fn battery(&self) -> Option<ArcStr> {
        None
    }

    fn locale(&self) -> Option<ArcStr> {
        env::var("LANG")
            .ok()
            .filter(|x| !x.is_empty())
            .or_else(|| env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or_else(|| env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
            .map(ArcStr::from)
    }

    fn theme(&self) -> Option<ArcStr> {
        None
    }

    fn icons(&self) -> Option<ArcStr> {
        None
    }

    fn displays(&self) -> Vec<ArcStr> {
        Vec::new()
    }

    fn machine(&self) -> Option<ArcStr> {
        None
    }

    fn kernel(&self) -> Option<ArcStr> {
        Some(ArcStr::from(self.uts.release().to_string_lossy()))
    }

    fn wm(&self) -> Option<ArcStr> {
        None
    }

    fn de(&self) -> Option<ArcStr> {
        None
    }

    fn shell(&self) -> Option<ArcStr> {
        None
    }

    fn cpu(&self) -> Option<ArcStr> {
        None
    }

    fn username(&self) -> Option<ArcStr> {
        unsafe {
            let uid = libc::getuid();
            let pwd = libc::getpwuid(uid);
            CStr::from_ptr((*pwd).pw_name)
                .to_str()
                .ok()
                .map(ArcStr::from)
        }
    }

    fn id(&self) -> ArcStr {
        arcstr::literal!("mac")
    }

    fn uptime(&self) -> Option<ArcStr> {
        unsafe {
            let time: *mut timespec = std::alloc::alloc(Layout::new::<timespec>()).cast();
            libc::clock_gettime(libc::CLOCK_UPTIME_RAW, time);
            Some(ArcStr::from(
                (
                    time::Duration::seconds(time.as_ref().unwrap().tv_sec)
                    // + time::Duration::nanoseconds(time.as_ref().unwrap().tv_nsec)
                )
                .to_string(),
            ))
        }
    }

    fn ip(&self) -> Vec<ArcStr> {
        Vec::new()
    }

    fn hostname(&self) -> Option<ArcStr> {
        self.uts.nodename().to_str().map(ArcStr::from)

    }
}
