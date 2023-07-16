#![cfg(target_family = "unix")]
use std::{
    alloc::Layout,
    ffi::{CStr, CString},
    fs,
    mem::{self, MaybeUninit},
    net::{Ipv4Addr, Ipv6Addr},
    sync::RwLock,
};

use crate::util::{bytecount_format, OSInfo};

use glob::glob;
use itertools::Itertools;
use libc::{getifaddrs, statvfs, AF_INET, AF_INET6, IFA_F_DEPRECATED, IFF_LOOPBACK, IFF_RUNNING};

use pci_ids::Device;
use platform_info::{PlatformInfo, PlatformInfoAPI};
use rayon::{
    prelude::{ParallelExtend, ParallelIterator},
    str::ParallelString,
};
use rustc_hash::{FxHashMap, FxHashSet};

pub struct LinuxInfo {
    uts: PlatformInfo,
    os_release: RwLock<FxHashMap<Box<str>, Box<str>>>,
}
impl LinuxInfo {
    pub fn new() -> Self {
        Self {
            uts: PlatformInfo::new().unwrap(),
            os_release: RwLock::default(),
        }
    }
}
impl OSInfo for LinuxInfo {
    fn os(&self) -> Option<String> {
        //Todo: check for lsb_release
        let data = fs::read_to_string("/etc/os-release").ok()?;
        self.os_release
            .write()
            .unwrap()
            .par_extend(data.par_lines().map(|line| {
                let (x, y) = line.split_once('=').unwrap();
                (
                    x.to_owned().into_boxed_str(),
                    y.trim_matches('"').to_owned().into_boxed_str(),
                )
            }));
        Some(
            (self.os_release).read().unwrap().get("NAME")?.to_string()
                + (self.os_release)
                    .read()
                    .unwrap()
                    .get("VERSION")
                    .map_or(String::new(), |x| " ".to_string() + x + " ")
                    .as_ref()
                + &self.uts.machine().to_string_lossy(),
        )
    }

    fn hostname(&self) -> Option<String> {
        Some(self.uts.nodename().to_string_lossy().to_string())
    }

    fn displays(&self) -> Vec<String> {
        || -> anyhow::Result<Vec<String>> {
            let mut res = Vec::new();
            let mut paths = glob("/sys/class/drm/card*-*/modes")?;
            while let Some(Ok(path)) = paths.next() {
                res.push(match fs::read_to_string(path)?.split_once('\n') {
                    Some(x) => x.0.to_owned(),
                    None => continue,
                });
            }
            Ok(res)
        }()
        .ok()
        .unwrap_or_default()
    }

    fn machine(&self) -> Option<String> {
        fs::read_to_string("/sys/class/dmi/id/product_name")
            .ok()
            .map(|x| x.trim().to_string())
    }

    fn kernel(&self) -> Option<String> {
        //.utsname.machine()
        Some(self.uts.release().to_string_lossy().to_string())
    }

    fn gpus(&self) -> Vec<String> {
        || -> anyhow::Result<Vec<String>> {
            let mut res = Vec::new();
            let mut paths = glob("/sys/class/drm/card?/device")?;
            while let Some(Ok(card)) = paths.next() {
                let path = card.to_str().unwrap().to_string() + "/vendor";
                let vid = u16::from_str_radix(&fs::read_to_string(path).unwrap().trim()[2..], 16)
                    .unwrap();

                let path = card.to_str().unwrap().to_string() + "/device";
                let pid = u16::from_str_radix(&fs::read_to_string(path).unwrap().trim()[2..], 16)
                    .unwrap();
                let device = &Device::from_vid_pid(vid, pid).unwrap();
                let vendor = device
                    .vendor()
                    .name()
                    .replace("Advanced Micro Devices, Inc. [AMD/ATI]", "AMD")
                    .replace("Intel Corporation", "Intel");
                res.push(vendor.clone() + " " + device.name());
            }
            Ok(res)
        }()
        .ok()
        .unwrap_or_default()
    }

    fn theme(&self) -> Option<String> {
        None
    }

    fn wm(&self) -> Option<String> {
        None
    }

    fn de(&self) -> Option<String> {
        None
    }

    fn shell(&self) -> Option<String> {
        let ppid = std::os::unix::process::parent_id();
        fs::read_to_string(format!("/proc/{ppid}/comm"))
            .ok()
            .map(|x| x.trim().to_string())
    }

    fn cpu(&self) -> Option<String> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
        let model = cpuinfo
            .lines()
            .find(|x| x.starts_with("model name"))?
            .split_once(':')?
            .1
            .trim();
        let cores = cpuinfo
            .lines()
            .find(|x| x.starts_with("cpu cores"))?
            .split_once(':')?
            .1
            .trim();

        let model = model.split_once('@')?;
        Some(model.0.to_string() + "(" + cores + ") @" + model.1)
    }

    fn username(&self) -> Option<String> {
        unsafe {
            let uid = libc::getuid();
            let pwd = libc::getpwuid(uid);
            CStr::from_ptr((*pwd).pw_name)
                .to_str()
                .map(std::string::ToString::to_string)
                .ok()
        }
    }

    fn sys_font(&self) -> Option<String> {
        None
    }

    fn cursor(&self) -> Option<String> {
        None
    }

    fn terminal(&self) -> Option<String> {
        None
    }

    fn term_font(&self) -> Option<String> {
        None
    }
    //todo: more decimal places
    fn memory(&self) -> Option<String> {
        let re = regex::Regex::new(r#"Mem(Total|Available):\W*(\d*)"#).unwrap();
        let mem = fs::read_to_string("/proc/meminfo").ok()?;
        let caps: (u64, u64) = re
            .captures_iter(&mem)
            .map(|x| str::parse::<u64>(x.get(2).unwrap().as_str()).unwrap())
            .collect_tuple()?;

        Some(format!(
            "{} / {}",
            bytecount_format((caps.0 - caps.1) << 10, 2),
            bytecount_format(caps.0 << 10, 2),
        ))
    }
    fn ip(&self) -> Vec<String> {
        let mut ipv4_addrs = FxHashSet::<Ipv4Addr>::default();
        let mut ipv6_addrs = FxHashSet::<Ipv6Addr>::default();
        unsafe {
            let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
            getifaddrs(addrs.as_mut_ptr());
            while let Some(addr) = addrs.assume_init().as_ref() {
                if addr.ifa_flags & IFF_RUNNING as u32 == 0 {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                }
                if addr.ifa_flags & IFF_LOOPBACK as u32 != 0 {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                }
                if addr.ifa_flags & IFA_F_DEPRECATED != 0 {
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
            ipv4_addrs.iter().fold(String::new(), |x, y| -> String {
                (if x.is_empty() { x } else { x + ", " }) + &y.to_string()
            }), /*,
                ipv6_addrs.iter().fold(String::new(), |x, y| {
                    (if x.is_empty() { x } else { x + ", " }) + &y.to_string()
                }),*/
        ]
    }
    fn disks(&self) -> Vec<(String, String)> {
        (|| -> Option<Vec<(String,String)>> {
            let mnt = fs::read_to_string("/proc/mounts").ok()?;
            let re = regex::Regex::new(r#"(^/dev/(loop|ram|fd))|(/var/snap)"#).unwrap();
            Some(mnt.par_lines()
            .filter_map(|line| -> Option<std::str::SplitAsciiWhitespace<'_>> {
                if re.is_match(line) {
                    return None;
                }

                if line.starts_with("/rpool/") || line.starts_with("drvfs") {
                    return Some(line.split_ascii_whitespace());
                }
                if !line.starts_with("/dev/") {
                    return None;
                }
                return Some(line.split_ascii_whitespace());
            })
            .filter_map(|mut x| -> Option<(String, String)> {
                let (Some(_name), Some(mount), Some(_filesystemm)) = (x.next(), x.next(), x.next()) else {
                    return None;
                };
                unsafe {
                    let buf: *mut statvfs = std::alloc::alloc(Layout::new::<statvfs>()).cast();

                    statvfs(CString::new(mount).ok().unwrap().as_ptr(), buf);
                    let total = (*buf).f_blocks ;
                    let size_used = total - ((*buf).f_bavail );
                    let block_size = (*buf).f_bsize;
                    if size_used == 0 {
                        return None;
                    }
                Some((format!("Disk ({mount})"), format!("{}/ {}", bytecount_format(size_used * block_size ,0),bytecount_format(total * block_size,0))))
                }
            }).collect::<Vec<(String,String)>>())
        })().unwrap_or_default()
    }

    fn battery(&self) -> Option<String> {
        None //todo: need to check /sys/class/power_supply on a laptop
    }

    fn locale(&self) -> Option<String> {
        std::env::var("LANG")
            .ok()
            .filter(|x| !x.is_empty())
            .or_else(|| std::env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
    }
    fn uptime(&self) -> Option<String> {
        None
    }
    fn icons(&self) -> Option<String> {
        None
    }
    fn id(&self) -> Box<str> {
        self.os_release
            .read()
            .unwrap()
            .get("ID")
            .unwrap().clone()
    }
}
