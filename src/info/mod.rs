use std::{
    str::FromStr,
    sync::{mpsc::Sender, Arc},
};

use arcstr::ArcStr;
use crossterm::style::{Color, Stylize};

#[cfg(target_os = "ios")]
use crate::info::iosinfo::IosInfo as get_info;
#[cfg(target_os = "linux")]
use crate::info::linuxinfo::LinuxInfo as get_info;
#[cfg(target_os = "macos")]
use crate::info::macinfo::MacInfo as get_info;
#[cfg(target_family = "windows")]
use crate::info::wininfo::WindowsInfo as get_info;
pub mod iosinfo;
pub mod linuxinfo;
pub mod macinfo;
pub mod wininfo;
pub trait OSInfo: Send + Sync {
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
        None
    }
    fn theme(&self) -> Option<ArcStr> {
        None
    }
    fn icons(&self) -> Option<ArcStr> {
        None
    }
    fn os(&self) -> Option<ArcStr> {
        None
    }
    fn id(&self) -> ArcStr;
    fn uptime(&self) -> Option<ArcStr>;
    fn ip(&self) -> Vec<ArcStr>;
    fn displays(&self) -> Vec<ArcStr> {
        Vec::new()
    }

    fn hostname(&self) -> Option<ArcStr>;

    fn machine(&self) -> Option<ArcStr> {
        None
    }

    fn kernel(&self) -> Option<ArcStr> {
        None
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
        None
    }
}

// #[derive(Debug)]
// pub struct Info {
//     pub os: Option<ArcStr>,
//     pub machine: Option<ArcStr>,
//     pub kernel: Option<ArcStr>,
//     pub uptime: Option<ArcStr>,
//     pub username: Option<ArcStr>,
//     pub hostname: Option<ArcStr>,
//     pub resolution: Vec<ArcStr>,
//     pub wm: Option<ArcStr>,
//     pub de: Option<ArcStr>,
//     pub shell: Option<ArcStr>,
//     pub cpu: Option<ArcStr>,
//     pub font: Option<ArcStr>,
//     pub cursor: Option<ArcStr>,
//     pub terminal: Option<ArcStr>,
//     pub terminal_font: Option<ArcStr>,
//     pub gpus: Vec<ArcStr>,
//     pub memory: Option<ArcStr>,
//     pub disks: Vec<(ArcStr, ArcStr)>,
//     pub battery: Option<ArcStr>,
//     pub locale: Option<ArcStr>,
//     pub theme: Option<ArcStr>,
//     pub icons: Option<ArcStr>,
//     pub ip: Vec<ArcStr>,
//     pub id: ArcStr,
// }

#[must_use]
pub fn get_id() -> ArcStr {
    get_info::new().id()
}

pub fn get_async(tx: Sender<(ArcStr, ArcStr)>) {
    let getter = Arc::new(get_info::new());
    let username = getter.username().unwrap_or_default();
    let hostname = getter.hostname().unwrap_or_default();
    let y = arcstr::format!("{username}@{hostname}");
    let repeats =
        ArcStr::from_str(std::str::from_utf8(vec![b'-'; y.len()].as_slice()).unwrap()).unwrap();
    tx.send((y, ArcStr::new())).unwrap();
    tx.send((repeats, ArcStr::new())).unwrap();
    rayon::scope(|s| {
        // general_readout: general_readout.clone(),
        s.spawn(|_| {
            getter
                .battery()
                .and_then(|e| tx.send((arcstr::literal!("Battery"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .cpu()
                .and_then(|e| tx.send((arcstr::literal!("cpu"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .cursor()
                .and_then(|e| tx.send((arcstr::literal!("cursor"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .de()
                .and_then(|e| tx.send((arcstr::literal!("de"), e)).ok());
        });
        s.spawn(|_| {
            for e in getter.disks() {
                tx.send(e).ok();
            }
        });
        s.spawn(|_| {
            getter
                .sys_font()
                .and_then(|e| tx.send((arcstr::literal!("sys_font"), e)).ok());
        });
        s.spawn(|_| {
            for (idx, e) in getter.gpus().into_iter().enumerate() {
                tx.send((arcstr::format!("GPU {}", idx + 1), e)).ok();
            }
        });
        s.spawn(|_| {
            getter
                .hostname()
                .and_then(|e| tx.send((arcstr::literal!("hostname"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .icons()
                .and_then(|e| tx.send((arcstr::literal!("icons"), e)).ok());
        });
        s.spawn(|_| {
            for e in getter.ip() {
                tx.send((arcstr::literal!("IP"), e)).ok();
            }
        });
        s.spawn(|_| {
            getter
                .kernel()
                .and_then(|e| tx.send((arcstr::literal!("kernel"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .locale()
                .and_then(|e| tx.send((arcstr::literal!("locale"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .machine()
                .and_then(|e| tx.send((arcstr::literal!("machine"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .memory()
                .and_then(|e| tx.send((arcstr::literal!("memory"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .os()
                .and_then(|e| tx.send((arcstr::literal!("OS"), e)).ok());
        });
        s.spawn(|_| {
            for (idx, e) in getter.displays().into_iter().enumerate() {
                tx.send((arcstr::format!("Display {}", idx + 1), e)).ok();
            }
        });
        s.spawn(|_| {
            getter
                .shell()
                .and_then(|e| tx.send((arcstr::literal!("Shell"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .terminal()
                .and_then(|e| tx.send((arcstr::literal!("Terminal"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .term_font()
                .and_then(|e| tx.send((arcstr::literal!("Term_font"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .theme()
                .and_then(|e| tx.send((arcstr::literal!("Theme"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .uptime()
                .and_then(|e| tx.send((arcstr::literal!("Uptime"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .username()
                .and_then(|e| tx.send((arcstr::literal!("Username"), e)).ok());
        });
        s.spawn(|_| {
            getter
                .wm()
                .and_then(|e| tx.send((arcstr::literal!("WM"), e)).ok());
        });
    });
    let (dark, light) = palette();
    tx.send((ArcStr::new(), dark)).ok();
    tx.send((ArcStr::new(), light)).ok();
}

fn palette() -> (ArcStr, ArcStr) {
    (
        (0..8u8)
            .map(|x| "   ".on(Color::AnsiValue(x)).to_string())
            .collect::<String>()
            .into(),
        (8..16u8)
            .map(|x| "   ".on(Color::AnsiValue(x)).to_string())
            .collect::<String>()
            .into(),
    )
}
