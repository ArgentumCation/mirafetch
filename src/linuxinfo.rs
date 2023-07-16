#![cfg(target_family = "unix")]
use std::{
    alloc::Layout,
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    fs,
    mem::{self, MaybeUninit},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::Deref,
};

use crate::util::{bytecount_format, OSInfo};

use glob::glob;
use itertools::Itertools;
use libc::{
    getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, statvfs, AF_INET, AF_INET6, IFA_F_DEPRECATED,
    IFA_F_TEMPORARY, IFF_LOOPBACK, IFF_RUNNING,
};

use pci_ids::Device;
use platform_info::{PlatformInfo, PlatformInfoAPI};
use rayon::{
    prelude::{FromParallelIterator, ParallelBridge, ParallelIterator},
    str::ParallelString,
};

pub struct LinuxInfo {
    uts: PlatformInfo,
}
impl LinuxInfo {
    pub fn new() -> Self {
        Self {
            uts: PlatformInfo::new().unwrap(),
        }
    }
}
impl OSInfo for LinuxInfo {
    fn os(&self) -> Option<String> {
        //Todo: check for lsb_release
        let data = fs::read_to_string("/etc/os-release").ok()?;

        let binding = regex::Regex::new(r#"(?P<key>.*)="?(?P<val>.*?)("|$)"#).ok()?;
        let captures = binding.captures_iter(&data);
        let os: HashMap<&str, &str> = HashMap::from_par_iter(captures.par_bridge().map(|x| {
            let k = x.name("key").unwrap().as_str();
            let v = x.name("val").unwrap().as_str();
            (k, v)
        }));
        Some(
            os["NAME"].deref().to_string()
                + os.get("VERSION")
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
                println!("{path}");
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
                res.push(vendor.to_owned() + " " + device.name());
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

        println!("{caps:#?}");
        Some(format!(
            "{} / {}",
            bytecount_format((caps.0 - caps.1) << 10),
            bytecount_format(caps.0 << 10),
        ))
    }
    fn ip(&self) -> Vec<String> {
        let mut res = HashSet::new();
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
                if addr.ifa_flags & IFA_F_DEPRECATED as u32 != 0 {
                    addrs = MaybeUninit::new(addr.ifa_next);
                    continue;
                }
                if (*addr.ifa_addr).sa_family as i32 == AF_INET {
                    let ipv4 = (*((addr.ifa_addr) as *mut sockaddr_in))
                        .sin_addr
                        .s_addr
                        .swap_bytes();
                    res.insert(Ipv4Addr::from(ipv4).to_string());
                }
                if (*addr.ifa_addr).sa_family as i32 == AF_INET6 {
                    let ipv6 = (*((addr.ifa_addr) as *mut sockaddr_in6)).sin6_addr.s6_addr;
                    if !ipv6.starts_with(&[0xfe, 0x80]) {
                        res.insert(Ipv6Addr::from(ipv6).to_string());
                    }
                }
                // if addr.ifa_next.is_null() {
                //     break;
                // }
                addrs = MaybeUninit::new(addr.ifa_next);
            }
        };
        // println!("{res:?}");
        res.into_iter().collect_vec()
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
                    let total = (*buf).f_blocks * (*buf).f_frsize;
                    let size_used = total - ((*buf).f_bavail * (*buf).f_frsize);
                    if size_used == 0 {
                        return None;
                    }
                Some((format!("Disk ({mount})"), format!("{}/ {}", bytecount_format(size_used),bytecount_format(total))))
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
            .or(std::env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or(std::env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
    }
    fn uptime(&self) -> Option<String> {
        None
    }
    fn icons(&self) -> Option<String> {
        None
    }
}
