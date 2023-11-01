use std::{fmt::Display, sync::Arc};

use arcstr::ArcStr;
use crossterm::style::{Color, Stylize};

#[cfg(target_os = "ios")]
use crate::info::iosinfo::IosInfo as get_info;
#[cfg(target_os = "linux")]
use crate::info::linuxinfo::LinuxInfo as get_info;
#[cfg(target_family = "windows")]
use crate::info::wininfo::WindowsInfo as get_info;
pub mod iosinfo;
pub mod linuxinfo;
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

#[derive(Debug)]
pub struct Info {
    pub os: Option<ArcStr>,
    pub machine: Option<ArcStr>,
    pub kernel: Option<ArcStr>,
    pub uptime: Option<ArcStr>,
    pub username: Option<ArcStr>,
    pub hostname: Option<ArcStr>,
    pub resolution: Vec<ArcStr>,
    pub wm: Option<ArcStr>,
    pub de: Option<ArcStr>,
    pub shell: Option<ArcStr>,
    pub cpu: Option<ArcStr>,
    pub font: Option<ArcStr>,
    pub cursor: Option<ArcStr>,
    pub terminal: Option<ArcStr>,
    pub terminal_font: Option<ArcStr>,
    pub gpus: Vec<ArcStr>,
    pub memory: Option<ArcStr>,
    pub disks: Vec<(ArcStr, ArcStr)>,
    pub battery: Option<ArcStr>,
    pub locale: Option<ArcStr>,
    pub theme: Option<ArcStr>,
    pub icons: Option<ArcStr>,
    pub ip: Vec<ArcStr>,
    pub id: ArcStr,
}

impl Default for Info {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        // let mut sys = System::new_all();
        let getter = Arc::new(get_info::new());
        let _getter_clone = Arc::clone(&getter);
        let mut battery = Default::default();
        let mut cpu = Default::default();
        let mut cursor = Default::default();
        let mut de = Default::default();
        let mut disks = Default::default();
        let mut font = Default::default();
        let mut gpus = Default::default();
        let mut hostname = Default::default();
        let mut icons = Default::default();
        let mut id: ArcStr = Default::default();
        let mut ip = Default::default();
        let mut kernel = Default::default();
        let mut locale = Default::default();
        let mut machine = Default::default();
        let mut memory = Default::default();
        let mut os = Default::default();
        let mut resolution = Default::default();
        let mut shell = Default::default();
        let mut terminal_font = Default::default();
        let mut terminal = Default::default();
        let mut theme = Default::default();
        let mut uptime = Default::default();
        let mut username = Default::default();
        let mut wm = Default::default();
        rayon::scope(|s| {
            // general_readout: general_readout.clone(),
            (*s).spawn(|_| battery = getter.battery());
            s.spawn(|_| cpu = getter.cpu());
            s.spawn(|_| cursor = getter.cursor());
            s.spawn(|_| de = getter.de());
            s.spawn(|_| disks = getter.disks());
            s.spawn(|_| font = getter.sys_font());
            s.spawn(|_| gpus = getter.gpus());
            s.spawn(|_| hostname = getter.hostname());
            s.spawn(|_| icons = getter.icons());
            s.spawn(|_| id = getter.id());
            s.spawn(|_| ip = getter.ip());
            s.spawn(|_| kernel = getter.kernel());
            s.spawn(|_| locale = getter.locale());
            s.spawn(|_| machine = getter.machine());
            s.spawn(|_| memory = getter.memory());
            s.spawn(|_| os = getter.os());
            s.spawn(|_| resolution = getter.displays());
            s.spawn(|_| shell = getter.shell());
            s.spawn(|_| terminal = getter.terminal());
            s.spawn(|_| terminal_font = getter.term_font());
            s.spawn(|_| theme = getter.theme());
            s.spawn(|_| uptime = getter.uptime());
            s.spawn(|_| username = getter.username());
            s.spawn(|_| wm = getter.wm());
        });

        Self {
            os,
            machine,
            kernel,
            uptime,
            username,
            hostname,
            resolution,
            wm,
            de,
            shell,
            cpu,
            font,
            cursor,
            terminal,
            terminal_font,
            gpus,
            memory,
            disks,
            battery,
            locale,
            theme,
            icons,
            ip,
            id,
        }
    }
}

impl Info {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn as_vec(self) -> Vec<(ArcStr, ArcStr)> {
        let username = self.username.unwrap_or_default();
        let hostname = self.hostname.unwrap_or_default();
        let y = arcstr::format!("{username}@{hostname}");
        let repeats = y.len();
        let (dark, light) = palette();
        let mut res: Vec<(ArcStr, ArcStr)> = vec![
            (y, Some(ArcStr::default())),
            (
                ArcStr::from(["-"].repeat(repeats).join("")),
                Some(ArcStr::new()),
            ),
            // if none, empty string
            // if not none
            (arcstr::literal!("OS"), self.os),
            (arcstr::literal!("Host"), self.machine),
            (arcstr::literal!("Kernel"), self.kernel),
            (arcstr::literal!("Uptime"), self.uptime),
            (arcstr::literal!("Shell"), self.shell),
            (arcstr::literal!("WM"), self.wm),
            (arcstr::literal!("DE"), self.de),
            (arcstr::literal!("CPU"), self.cpu),
            (arcstr::literal!("Theme"), self.theme),
            (arcstr::literal!("System Font"), self.font),
            (arcstr::literal!("Cursor"), self.cursor),
            (arcstr::literal!("Terminal"), self.terminal),
            (arcstr::literal!("Terminal Font"), self.terminal_font),
            (arcstr::literal!("Memory"), self.memory),
            (arcstr::literal!("Battery"), self.battery),
            (arcstr::literal!("Locale"), self.locale),
            (arcstr::literal!("Icon Theme"), self.icons),
        ]
        .into_iter()
        .filter_map(|(x, y)| y.map(|z| (x, z)))
        .chain(
            self.resolution
                .into_iter()
                .enumerate()
                .map(|(idx, res)| (arcstr::format!("Display {}", idx + 1), res)),
        )
        .chain(
            self.gpus
                .into_iter()
                .enumerate()
                .map(|(idx, res)| (arcstr::format!("GPU {}", idx + 1), res)),
        )
        .chain(self.disks)
        .chain(self.ip.into_iter().map(|x| (arcstr::literal!("IP"), x)))
        .collect();
        res.push((ArcStr::new(), dark));
        res.push((ArcStr::new(), light));
        res
    }

    /// TODO
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    #[allow(dead_code)]
    fn info_fmt(
        f: &mut std::fmt::Formatter<'_>,
        info_type: &str,
        val: Option<&ArcStr>,
    ) -> std::fmt::Result {
        if let Some(x) = val {
            info_type
                .with(Color::Red)
                .attribute(crossterm::style::Attribute::Bold)
                .fmt(f)?;
            x.clone().reset().fmt(f)?;
            '\n'.fmt(f)?;
        }
        Ok(())
    }
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
