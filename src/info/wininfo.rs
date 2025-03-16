#![cfg(target_family = "windows")]

// use crate::Color;
use arcstr::ArcStr;
use crossterm::style::{Color, Stylize};
use itertools::Itertools;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use std::mem::size_of;
use std::net::{Ipv4Addr, Ipv6Addr};
use windows::Win32::System::SystemInformation::{
    RelationProcessorCore, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
};
use windows::Win32::{
    Foundation::ERROR_BUFFER_OVERFLOW,
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
        GAA_FLAG_SKIP_MULTICAST, IP_ADAPTER_ADDRESSES_LH,
    },
    Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6},
    System::SystemInformation::{GetLogicalProcessorInformationEx, RelationAll},
};
use winsafe::co::{SPI, SPIF};
use winsafe::{
    GetComputerName, GetDiskFreeSpaceEx, GetLogicalDriveStrings, GetTickCount64,
    GlobalMemoryStatusEx, SystemParametersInfo, MEMORYSTATUSEX, NONCLIENTMETRICS,
};
use wmi::{COMLibrary, WMIConnection};

use regex::Regex;
use std::sync::{Arc, OnceLock};

use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::info::OSInfo;
use crate::util::bytecount_format;

#[derive(Default)]
pub struct WindowsInfo {
    hklm: OnceLock<RegKey>,
}
impl WindowsInfo {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hklm: OnceLock::new(),
        }
    }

    fn get_hklm(&self) -> &RegKey {
        self.hklm.get_or_init(|| RegKey::predef(HKEY_LOCAL_MACHINE))
    }
}
impl OSInfo for WindowsInfo {
    fn displays(&self) -> Vec<ArcStr> {
        use std::sync::Mutex;

        use winsafe::{
            prelude::{user_Hdc, Handle},
            HDC,
        };
        let displays = Arc::new(Mutex::new(Vec::new()));
        let handle = HDC::NULL;
        handle
            .EnumDisplayMonitors(None, |_monitor, _hdc, rect| -> bool {
                displays.lock().unwrap().push(arcstr::format!(
                    "{}x{}",
                    rect.right - rect.left,
                    rect.bottom - rect.top
                ));
                true
            })
            .ok();
        return displays.lock().unwrap().to_vec();
    }

    fn machine(&self) -> Option<ArcStr> {
        self.get_hklm();
        let subkey = self
            .get_hklm()
            .open_subkey(r"HARDWARE\DESCRIPTION\System\BIOS")
            .ok()?;
        let res = arcstr::format!(
            "{} ({})",
            subkey
                .get_value::<String, &str>("SystemProductName")
                .ok()
                .unwrap_or_default(),
            subkey
                .get_value::<String, &str>("SystemFamily")
                .ok()
                .unwrap_or_default() // subkey.get_value::<ArcStr, &str>("SystemSKU").ok().unwrap_or_default()
                                     // subkey.get_value::<ArcStr, &str>("SystemManufacturer").ok().unwrap_or_default()
        );
        Some(res)
    }

    fn theme(&self) -> Option<ArcStr> {
        let binding = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes")
            .ok()?
            .get_value::<String, &str>("CurrentTheme")
            .ok()?;
        let re = Regex::new(r".*\\(.*)\.").unwrap();
        let theme_name: &str = re.captures_iter(binding.as_str()).next()?.get(1)?.as_str();
        let dwm = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"Software\Microsoft\Windows\DWM")
            .ok()?;
        let color: u32 = (0x00ff_ffff
            & dwm
                .get_value::<u32, &str>("AccentColor")
                .or_else(|_| dwm.get_value::<u32, &str>("ColorizationColor"))
                .ok()?)
        .swap_bytes()
            >> 8;
        Some(ArcStr::from(
            format!("{theme_name} (#{color:X})")
                .on(Color::Rgb {
                    r: (color >> 16) as u8,
                    g: ((color >> 8) & 0xff) as u8,
                    b: (color & 0xff) as u8,
                })
                .to_string(),
        ))
    }

    fn kernel(&self) -> Option<ArcStr> {
        let current_version = self
            .get_hklm()
            .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            .ok()?;
        let major: u32 = current_version
            .get_value("CurrentMajorVersionNumber")
            .ok()?;
        let minor: u32 = current_version
            .get_value("CurrentMinorVersionNumber")
            .ok()?;
        let build_number: ArcStr = current_version
            .get_value::<String, &str>("CurrentBuildNumber")
            .ok()?
            .into();
        let version: ArcStr = current_version
            .get_value::<String, &str>("DisplayVersion")
            .ok()?
            .into();
        let ubr: u32 = current_version.get_value("UBR").ok()?;
        Some(arcstr::format!(
            "{major}.{minor}.{build_number}.{ubr} ({version})"
        ))
    }

    fn wm(&self) -> Option<ArcStr> {
        if winsafe::DwmIsCompositionEnabled().ok()? {
            return Some(arcstr::literal!("Desktop Window Manager"));
        }
        Some(arcstr::literal!("Internal"))
    }

    fn de(&self) -> Option<ArcStr> {
        let current_version = self
            .get_hklm()
            .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            .ok()?;
        let major: u32 = current_version
            .get_value("CurrentMajorVersionNumber")
            .ok()?;
        let minor: u32 = current_version
            .get_value("CurrentMinorVersionNumber")
            .ok()?;
        if major >= 10 {
            return Some(arcstr::literal!("Fluent"));
        } else if major >= 6 {
            if minor >= 2 {
                return Some(arcstr::literal!("Metro"));
            }
            return Some(arcstr::literal!("Aero"));
        }
        None
    }

    fn gpus(&self) -> Vec<ArcStr> {
        || -> Option<Vec<ArcStr>> {
            let video: RegKey = self
                .get_hklm()
                .open_subkey(r"SYSTEM\CurrentControlSet\Control\Video\")
                .ok()?;

            Some(
                video
                    .enum_keys()
                    .filter_map(|x| {
                        video
                            .open_subkey(x.ok()?)
                            .map(|uuid| {
                                uuid.enum_keys().find_map(|y| -> Option<ArcStr> {
                                    uuid.open_subkey(y.unwrap())
                                        .ok()?
                                        .get_value::<String, &str>("DriverDesc")
                                        .ok()
                                        .map(ArcStr::from)
                                })
                                // .collect::<ArcStr>()
                            })
                            .ok()?
                    })
                    .collect_vec(),
            )
        }()
        .unwrap_or_default()
    }

    fn id(&self) -> ArcStr {
        || -> Option<ArcStr> {
            let com_con = COMLibrary::new().ok()?;
            let wmi_con = WMIConnection::new(com_con).ok()?;
            let binding = wmi_con
                .raw_query::<FxHashMap<String, String>>("SELECT Caption FROM Win32_OperatingSystem")
                .ok()?;
            Some(ArcStr::from(
                binding
                    .first()?
                    .values()
                    .next()?
                    .trim_start_matches("Microsoft ")
                    .split_ascii_whitespace()
                    .dropping_back(1)
                    .join(" "),
            ))
        }()
        .unwrap()
    }

    fn uptime(&self) -> Option<ArcStr> {
        let uptime = time::Duration::milliseconds(GetTickCount64().try_into().ok()?).to_string();
        Some(
            uptime
                .split_inclusive('m')
                .next()?
                .replace('h', " hours, ")
                .replace('m', " mins")
                .into(),
        )
    }

    fn ip(&self) -> Vec<ArcStr> {
        unsafe {
            let size = Box::into_raw(Box::new(0x3FFF));
            let mut buf = Vec::<u8>::with_capacity(*size as usize);
            let addrs: *mut IP_ADAPTER_ADDRESSES_LH = buf.as_mut_ptr().cast();
            while GetAdaptersAddresses(
                u32::from(AF_UNSPEC.0),
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                None,
                Some(addrs),
                size,
            ) == ERROR_BUFFER_OVERFLOW.0
            {
                buf.reserve_exact(*size as usize);
            }
            let mut ipv4_addrs: Vec<Ipv4Addr> = Vec::new();
            let mut ipv6_addrs: Vec<Ipv6Addr> = Vec::new();
            let mut adapter_current = addrs.as_ref();
            let mut res = Vec::new();
            while let Some(adapter) = adapter_current {
                let mut addr_current = (adapter).FirstUnicastAddress.as_ref();
                while let Some(addr) = addr_current {
                    match (*addr.Address.lpSockaddr).sa_family {
                        AF_INET6 => {
                            let ip_addr = Ipv6Addr::from(
                                (*addr.Address.lpSockaddr.cast::<SOCKADDR_IN6>())
                                    .sin6_addr
                                    .u
                                    .Byte,
                            );
                            if !ip_addr.is_loopback() && (ip_addr.segments()[0] & 0xfe00 == 0xfc00)
                            {
                                ipv6_addrs.push(ip_addr);
                            }
                        }
                        AF_INET => {
                            #[allow(clippy::cast_ptr_alignment)]
                            let ip_addr = Ipv4Addr::from(
                                (*addr.Address.lpSockaddr.cast::<SOCKADDR_IN>())
                                    .sin_addr
                                    .S_un
                                    .S_addr
                                    .swap_bytes(),
                            );
                            if !ip_addr.is_loopback()
                                && (ip_addr.is_private()/*|| ip_addr.is_link_local()*/)
                            {
                                ipv4_addrs.push(ip_addr);
                            }
                        }
                        _ => panic!(),
                    };
                    res.push(());
                    addr_current = addr.Next.as_ref();
                }
                adapter_current = adapter.Next.as_ref();
            }
            let mut res = Vec::with_capacity(2);
            if !ipv4_addrs.is_empty() {
                res.push(ArcStr::from(
                    ipv4_addrs
                        .par_iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                ));
            }
            if !ipv6_addrs.is_empty() {
                res.push(ArcStr::from(
                    ipv6_addrs
                        .par_iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                ));
            }

            res
        }
    }

    fn hostname(&self) -> Option<ArcStr> {
        GetComputerName()
            .ok()
            .map(|f| ArcStr::from(f.to_lowercase()))
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sys_font(&self) -> Option<ArcStr> {
        let mut metrics = NONCLIENTMETRICS::default();
        let size = size_of::<NONCLIENTMETRICS>();
        unsafe {
            SystemParametersInfo(
                SPI::GETNONCLIENTMETRICS,
                size as u32,
                &mut metrics,
                SPIF::NoValue,
            )
            .unwrap();
        };
        Some(metrics.lfMenuFont.lfFaceName().into())
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

    fn memory(&self) -> Option<ArcStr> {
        let result = GlobalMemoryStatusEx().ok()?;
        Some(arcstr::format!(
            "{} / {}",
            bytecount_format(result.ullTotalPhys - result.ullAvailPhys, 2),
            bytecount_format(result.ullTotalPhys, 2),
        ))
    }

    fn disks(&self) -> Vec<(ArcStr, ArcStr)> {
        let q = GetLogicalDriveStrings();
        q.map_or(Vec::new(), |c| {
            c.par_iter()
                .filter_map(|x| {
                    Some((ArcStr::from(x), {
                        let var_name = 0xDEAD;
                        let mut total: Option<u64> = Some(var_name);
                        let var_name = 0xDEAD;
                        let mut free: Option<u64> = Some(var_name);
                        GetDiskFreeSpaceEx(Some(x), None, total.as_mut(), free.as_mut()).ok()?;
                        arcstr::format!(
                            "{} / {}",
                            bytecount_format(total? - free?, 0),
                            bytecount_format(total?, 0)
                        )
                    }))
                })
                .collect()
        })
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

    fn icons(&self) -> Option<ArcStr> {
        None
    }

    fn os(&self) -> Option<ArcStr> {
        let com_con = COMLibrary::new().ok()?;
        let wmi_con = WMIConnection::new(com_con).ok()?;
        let binding = wmi_con
            .raw_query::<FxHashMap<String, String>>("SELECT Caption FROM Win32_OperatingSystem")
            .ok()?;
        binding.first()?.values().next().map(ArcStr::from)
    }

    fn shell(&self) -> Option<ArcStr> {
        None
        // let pid = get_current_pid().ok()?;
        // let parent_pid = s.process(pid)?.parent()?;
        // let parent = s.process(parent_pid)?.name();
        // Some(parent.replace(".exe", ""))
    }

    fn cpu(&self) -> Option<ArcStr> {
        let mut length = 0;
        let mut buf: Vec<u8>;
        let mut core_count = 0;
        unsafe {
            GetLogicalProcessorInformationEx(RelationAll, None, &mut length);
            buf = Vec::<u8>::with_capacity(length as usize);

            GetLogicalProcessorInformationEx(
                RelationAll,
                Some(buf.as_mut_ptr().cast()),
                &mut length,
            );
            let mut current: *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX = buf.as_mut_ptr().cast();
            while current.cast() < buf.as_mut_ptr().add(length as usize) {
                if (*current).Relationship == RelationProcessorCore {
                    core_count += 1;
                }
                //todo: logical/online cores
                current = current.cast::<u8>().add((*current).Size as usize).cast();
            }
        };
        let core0 = self
            .get_hklm()
            .open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0")
            .ok()?;
        let name: ArcStr = core0
            .get_value::<String, &str>("ProcessorNameString")
            .ok()?
            .into();
        let freq: u32 = core0.get_value("~MHz").ok()?;
        Some(arcstr::format!(
            "{name} ({core_count}) @ {:.2}GHz",
            freq as f32 / 1000.0
        ))
    }

    fn username(&self) -> std::option::Option<ArcStr> {
        winsafe::GetUserName().ok().map(ArcStr::from)
    }
}
