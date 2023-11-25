#[cfg(target_os = "macos")]
use arcstr::ArcStr;

#[cfg(target_os = "macos")]
use sysctl::Sysctl;

use platform_info::*;

use rustc_hash::{FxHashMap, FxHashSet};

use itertools::Itertools;

use std::{sync::OnceLock, alloc::Layout, ffi::{CStr, CString}, fs, mem::{self, MaybeUninit}, net::{Ipv4Addr, Ipv6Addr}};

use libc::timespec;

use crate::info::OSInfo;

pub struct MacInfo {
    uts: PlatformInfo
}

impl MacInfo {
    pub fn new() -> Self {
        MacInfo {
            uts: PlatformInfo::new().unwrap()
        }
    }
}

impl OSInfo for MacInfo {
    fn os(&self) -> Option<ArcStr> {
       None 
    }

    fn hostname(&self) -> Option<ArcStr> {
       Some(ArcStr::from(whoami::hostname()))
    }

    fn displays(&self) -> Vec<ArcStr> {
       vec![]
    }

    fn machine(&self) -> Option<ArcStr> {
       None 
    }

    fn kernel(&self) -> Option<ArcStr> {
        Some(ArcStr::from(self.uts.release().to_string_lossy()))
    }

    #[allow(clippy::similar_names)]
    fn gpus(&self) -> Vec<ArcStr> {
       vec![]
    }

    // TODO
    fn theme(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn wm(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn de(&self) -> Option<ArcStr> {
        None
    }

    fn shell(&self) -> Option<ArcStr> {
       None 
    }

    fn cpu(&self) -> Option<ArcStr> {
       let model = sysctl::Ctl::new("machdep.cpu.brand_string").unwrap().value_string().unwrap();
       let core_count = sysctl::Ctl::new("machdep.cpu.core_count").unwrap().value_string().unwrap();

       
       Some(arcstr::format!(
          "{} ({})",
          model,
          core_count
       ))
    }

    fn username(&self) -> Option<ArcStr> {
       Some(ArcStr::from(whoami::username()))
    }

    // TODO
    fn sys_font(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn cursor(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn terminal(&self) -> Option<ArcStr> {
        None
    }

    // TODO
    fn term_font(&self) -> Option<ArcStr> {
        None
    }

    fn memory(&self) -> Option<ArcStr> {
       None 
    }

    fn ip(&self) -> Vec<ArcStr> {
        use libc::{
            getifaddrs, statvfs, timespec, AF_INET, AF_INET6, IFF_LOOPBACK, IFF_RUNNING,
        };
        let mut ipv4_addrs = FxHashSet::<Ipv4Addr>::default();
        let mut ipv6_addrs = FxHashSet::<Ipv6Addr>::default();
        unsafe {
            let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
            getifaddrs(addrs.as_mut_ptr());
            while let Some(addr) = addrs.assume_init().as_ref() {
                if addr.ifa_addr.is_null() {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                }
                if addr.ifa_flags & IFF_RUNNING as u32 == 0 {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                }
                if addr.ifa_flags & IFF_LOOPBACK as u32 != 0 {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                } 
                if i32::from((*addr.ifa_addr).sa_family) == AF_INET {
                    let ipv4 = (*(addr.ifa_addr).cast::<libc::sockaddr_in>())
                        .sin_addr
                        .s_addr
                        .swap_bytes();
                    ipv4_addrs.insert(Ipv4Addr::from(ipv4));
                }
                if i32::from((*addr.ifa_addr).sa_family) == AF_INET6 {
                    let ipv6 = (*(addr.ifa_addr).cast::<libc::sockaddr_in6>())
                        .sin6_addr
                        .s6_addr;
                    if !ipv6.starts_with(&[0xfe, 0x80]) {
                        ipv6_addrs.insert(Ipv6Addr::from(ipv6));
                    }
                }
                // if addr.ifa_next.is_null() {
                //     break;
                // }
                addrs = MaybeUninit::new(addr.ifa_next);
            }
        };

        vec![
            ArcStr::from(
                ipv4_addrs
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect_vec()
                    .join(", "),
            ),
            /*ipv6_addrs.iter().fold(ArcStr::new(), |x, y| {
                (if x.is_empty() { x } else { x + ", " }) + &y.to_string()
            }),*/
        ]
    }

    fn disks(&self) -> Vec<(ArcStr, ArcStr)> { 
        vec![]
    }

    fn battery(&self) -> Option<ArcStr> {
        None 
    }

    fn locale(&self) -> Option<ArcStr> {
        std::env::var("LANG")
            .ok()
            .filter(|x| !x.is_empty())
            .or_else(|| std::env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
            .map(ArcStr::from)
    }

    fn uptime(&self) -> Option<ArcStr> {
        unsafe {
            let time: *mut timespec = std::alloc::alloc(Layout::new::<timespec>()).cast();
            libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, time);
            Some(ArcStr::from(
                (
                    time::Duration::seconds(time.as_ref().unwrap().tv_sec)
                    // + time::Duration::nanoseconds(time.as_ref().unwrap().tv_nsec)
                )
                .to_string(),
            ))
        }
    }

    // TODO
    fn icons(&self) -> Option<ArcStr> {
        None
    }

    fn id(&self) -> ArcStr {
       ArcStr::from("macos") 
    }
}
