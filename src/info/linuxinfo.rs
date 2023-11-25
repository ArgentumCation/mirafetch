#![cfg(target_os = "linux")]
use crate::info::OSInfo;
use crate::util::bytecount_format;
use anyhow::anyhow;
use arcstr::ArcStr;
use glob::glob;
use itertools::Itertools;
use libc::{
    getifaddrs, statvfs, timespec, AF_INET, AF_INET6, IFA_F_DEPRECATED, IFF_LOOPBACK, IFF_RUNNING,
};
use pci_ids::Device;
use platform_info::UNameAPI;
use platform_info::{PlatformInfo, PlatformInfoAPI};
use rayon::{
    prelude::{ParallelExtend, ParallelIterator},
    str::ParallelString,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::env;
use std::{
    alloc::Layout,
    ffi::{CStr, CString},
    fs,
    mem::{self, MaybeUninit},
    net::{Ipv4Addr, Ipv6Addr},
    sync::OnceLock,
};

pub struct LinuxInfo {
    uts: PlatformInfo,
    os_release: OnceLock<FxHashMap<ArcStr, ArcStr>>,
}

impl Default for LinuxInfo {
    fn default() -> Self {
        Self::new()
    }
}
impl LinuxInfo {
    #[must_use] pub fn new() -> Self {
        Self {
            uts: PlatformInfo::new().unwrap(),
            os_release: OnceLock::default(),
        }
    }

    fn os_release(&self) -> &FxHashMap<ArcStr, ArcStr> {
        self.os_release.get_or_init(|| {
            let mut res = FxHashMap::default();
            let data = fs::read_to_string("/etc/os-release").ok().unwrap();
            res.par_extend(data.par_lines().map(|line| {
                let (x, y) = line.split_once('=').unwrap();
                (
                    x.to_owned().into_boxed_str().into(),
                    y.trim_matches('"').to_owned().into_boxed_str().into(),
                )
            }));
            res
        })
    }
}
impl OSInfo for LinuxInfo {
    fn os(&self) -> Option<ArcStr> {
        // Base name
        let mut res = String::new();
        if let Some(name) = self.os_release().get("NAME") {
            res += name;
        } else if let Some(name) = self.os_release().get("PRETTY_NAME") {
            res += name;
        } else if let Some(name) = self.os_release().get("ID") {
            res += name;
        } else {
            return None;
        }

        // Codename
        if let Some(codename) = self.os_release().get("VERSION_ID") {
            if !res.contains(codename.as_str()) {
                res += " ";
                res += codename;
            }
        } else if let Some(codename) = self.os_release().get("VERSION_CODENAME") {
            if !res.contains(codename.as_str()) {
                res += " ";
                res += codename;
            }
        }

        // Version
        if let Some(version) = self.os_release().get("VERSION_ID") {
            if !res.contains(version.as_str()) {
                res += " ";
                res += version;
            }
        } else if let Some(version) = self.os_release().get("VERSION") {
            if !res.contains(version.as_str()) {
                res += " ";
                res += version;
            }
        }

        let arch = self.uts.machine();
        if !res.contains(arch.to_str().unwrap()) {
            res += " ";
            res += arch.to_str().unwrap();
        }
        Some(ArcStr::from(res.trim()))
    }

    fn hostname(&self) -> Option<ArcStr> {
        Some(ArcStr::from(self.uts.nodename().to_str()?))
    }

    fn displays(&self) -> Vec<ArcStr> {
        || -> anyhow::Result<Vec<ArcStr>> {
            let mut res = Vec::new();
            let mut paths = glob("/sys/class/drm/card*-*/modes")?;
            while let Some(Ok(path)) = paths.next() {
                res.push(match fs::read_to_string(path)?.split_once('\n') {
                    Some(x) => ArcStr::from(x.0),
                    None => continue,
                });
            }
            Ok(res)
        }()
        .ok()
        .unwrap_or_default()
    }

    fn machine(&self) -> Option<ArcStr> {
        fs::read_to_string("/sys/class/dmi/id/product_name")
            .ok()
            .or_else(|| fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").ok())
            .or_else(|| fs::read_to_string("/sys/firmware/devicetree/base/model").ok())
            .or_else(|| fs::read_to_string("/sys/firmware/devicetree/base/banner-name").ok())
            .or_else(|| fs::read_to_string("/sys/devices/virtual/dmi/id/product_family").ok())
            .or_else(|| fs::read_to_string("/sys/class/dmi/id/product_family").ok())
            .or_else(|| fs::read_to_string("/sys/devices/virtual/dmi/id/product_version").ok())
            .or_else(|| fs::read_to_string("/sys/class/dmi/id/product_version").ok())
            .or_else(|| fs::read_to_string("/sys/devices/virtual/dmi/id/product_sku").ok())
            .or_else(|| fs::read_to_string("/sys/class/dmi/id/product_sku").ok())
            .or_else(|| fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor").ok())
            .or_else(|| fs::read_to_string("/sys/class/dmi/id/sys_vendor").ok())
            .or_else(|| {
                env::var("WSL_DISTRO_NAME")
                    .or_else(|_| env::var("WSL_DISTRO"))
                    .or_else(|_| env::var("WSL_INTEROP"))
                    .ok()
                    .map(|_| "Windows Subsystem for Linux".to_string())
            })
            .map(|f| {
                if f.starts_with("Standard PC") {
                    String::from("KVM/QEMU") + &f
                } else {
                    f
                }
            })
            .map(|x| ArcStr::from(x.trim()))
    }

    fn kernel(&self) -> Option<ArcStr> {
        //.utsname.machine()
        Some(ArcStr::from(self.uts.release().to_string_lossy()))
    }

    #[allow(clippy::similar_names)]
    fn gpus(&self) -> Vec<ArcStr> {
        || -> anyhow::Result<Vec<ArcStr>> {
            let mut res: Vec<ArcStr> = Vec::new();
            let mut paths = glob("/sys/class/drm/card?/device")?;
            while let Some(Ok(card)) = paths.next() {
                let path = card.join("vendor");
                if !path.exists() {
                    continue;
                }
                let vid = u16::from_str_radix(&fs::read_to_string(path).unwrap().trim()[2..], 16)
                    .unwrap();

                let path = card.join("device");
                if !path.exists() {
                    continue;
                }
                let pid = u16::from_str_radix(&fs::read_to_string(path).unwrap().trim()[2..], 16)
                    .unwrap();
                let device = &Device::from_vid_pid(vid, pid).unwrap();
                let vendor = device
                    .vendor()
                    .name()
                    .replace("Advanced Micro Devices, Inc. [AMD/ATI]", "AMD")
                    .replace("Intel Corporation", "Intel");
                res.push(arcstr::format!("{vendor} {}", device.name()));
            }
            Ok(res)
        }()
        .ok()
        .unwrap_or_default()
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
        let de = env::var("XDG_CURRENT_DESKTOP").ok()?;
        let protocol = env::var("XDG_SESSION_TYPE").ok();
        Some(match protocol {
            Some(proto) => arcstr::format!("{de} ({proto})"),
            None => de.into(),
        })
    }

    fn shell(&self) -> Option<ArcStr> {
        let ppid = std::os::unix::process::parent_id();
        fs::read_to_string(format!("/proc/{ppid}/comm"))
            .ok()
            .map(|x| ArcStr::from(x.trim()))
    }

    fn cpu(&self) -> Option<ArcStr> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
        let model = cpuinfo
            .lines()
            .find(|x| x.starts_with("model name"))?
            .split_once(':')?
            .1
            .trim();
        let cores = cpuinfo
            .lines()
            .find(|x| x.starts_with("siblings"))?
            .split_once(':')?
            .1
            .trim();

        let freq: f32 = cpuinfo
            .lines()
            .find(|x| x.starts_with("cpu MHz"))?
            .split_once(':')?
            .1
            .trim()
            .parse()
            .ok()?;
        Some(arcstr::format!(
            "{} ({}) @ {:.2} GHz)",
            model,
            cores,
            freq / 1000.0
        ))
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
        let re = regex::Regex::new(r"Mem(Total|Available):\W*(\d*)").unwrap();
        let mem = fs::read_to_string("/proc/meminfo").ok()?;
        let caps: (u64, u64) = re
            .captures_iter(&mem)
            .map(|x| str::parse::<u64>(x.get(2).unwrap().as_str()).unwrap())
            .collect_tuple()?;

        Some(arcstr::format!(
            "{} / {}",
            bytecount_format((caps.0 - caps.1) << 10, 2),
            bytecount_format(caps.0 << 10, 2),
        ))
    }

    fn ip(&self) -> Vec<ArcStr> {
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
        (|| -> Option<Vec<(ArcStr, ArcStr)>> {
            let mnt = fs::read_to_string("/proc/mounts").ok()?;
            let re = regex::Regex::new(r"(^/dev/(loop|ram|fd))|(/var/snap)").unwrap();
            Some(
                mnt.par_lines()
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
                    .filter_map(|mut x| -> Option<(ArcStr, ArcStr)> {
                        let (Some(_name), Some(mount), Some(_filesystemm)) =
                            (x.next(), x.next(), x.next())
                        else {
                            return None;
                        };
                        unsafe {
                            let buf: *mut statvfs =
                                std::alloc::alloc(Layout::new::<statvfs>()).cast();

                            statvfs(CString::new(mount).ok().unwrap().as_ptr(), buf);
                            let total = (*buf).f_blocks;
                            let size_used = total.checked_sub((*buf).f_bavail)?;
                            let block_size = (*buf).f_bsize;
                            if size_used == 0 {
                                return None;
                            }
                            // println!("Size Used: {size_used}, Block Size {block_size}");
                            size_used.checked_mul(block_size).map(|bytes| {
                                (
                                    arcstr::format!("Disk ({mount})"),
                                    arcstr::format!(
                                        "{}/ {}",
                                        bytecount_format(bytes, 0),
                                        bytecount_format(total * block_size, 0)
                                    ),
                                )
                            })
                        }
                    })
                    .collect::<Vec<(ArcStr, ArcStr)>>(),
            )
        })()
        .unwrap_or_default()
    }

    fn battery(&self) -> Option<ArcStr> {
        //TODO refactor this into a vector
        Some(ArcStr::from(
            glob::glob("/sys/class/power_supply/BAT*/")
                .ok()?
                .filter_map(|x| {
                    x.map_err(|op| anyhow!(op))
                        .and_then(|path| {
                            let mut bat = fs::read_to_string(path.join("capacity"))?
                                .trim()
                                .to_string()
                                + "% ";
                            if let Ok(status) = fs::read_to_string(path.join("status")) {
                                if !status.contains("Unknown") {
                                    bat += &status;
                                }
                            }
                            Ok(bat.trim().to_string())
                        })
                        .ok()
                })
                .collect_vec()
                .join(", "),
        ))
    }

    fn locale(&self) -> Option<ArcStr> {
        env::var("LANG")
            .ok()
            .filter(|x| !x.is_empty())
            .or_else(|| env::var("LC_ALL").ok().filter(|x| !x.is_empty()))
            .or_else(|| env::var("LC_MESSAGES").ok().filter(|x| !x.is_empty()))
            .map(ArcStr::from)
    }

    fn uptime(&self) -> Option<ArcStr> {
        unsafe {
            let time: *mut timespec = std::alloc::alloc(Layout::new::<timespec>()).cast();
            libc::clock_gettime(libc::CLOCK_BOOTTIME, time);
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
        self.os_release();
        self.os_release().get("ID").unwrap().clone()
    }
}
