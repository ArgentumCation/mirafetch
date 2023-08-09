use std::{fmt::Display, sync::Arc};

use crossterm::style::{Color, Stylize};

#[cfg(target_os = "ios")]
use crate::info::iosinfo::iOSInfo as get_info;
#[cfg(target_os = "linux")]
use crate::info::linuxinfo::LinuxInfo as get_info;
#[cfg(target_family = "windows")]
use crate::info::wininfo::WindowsInfo as get_info;
pub mod iosinfo;
pub mod linuxinfo;
pub mod wininfo;
pub trait OSInfo: Send + Sync {
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
    fn gpus(&self) -> Vec<Arc<str>> {
        Vec::new()
    }
    fn memory(&self) -> Option<String> {
        None
    }
    fn disks(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    fn battery(&self) -> Option<String> {
        None
    }
    fn locale(&self) -> Option<String> {
        None
    }
    fn theme(&self) -> Option<String> {
        None
    }
    fn icons(&self) -> Option<String> {
        None
    }
    fn os(&self) -> Option<String> {
        None
    }
    fn id(&self) -> Arc<str>;
    fn uptime(&self) -> Option<String>;
    fn ip(&self) -> Vec<Arc<str>>;
    fn displays(&self) -> Vec<Arc<str>> {
        Vec::new()
    }

    fn hostname(&self) -> Option<Arc<str>>;

    fn machine(&self) -> Option<String> {
        None
    }

    fn kernel(&self) -> Option<String> {
        None
    }

    fn wm(&self) -> Option<String> {
        None
    }

    fn de(&self) -> Option<String> {
        None
    }

    fn shell(&self) -> Option<String> {
        None
    }
    fn cpu(&self) -> Option<String> {
        None
    }

    fn username(&self) -> Option<Arc<str>> {
        None
    }
}

#[derive(Debug)]
pub struct Info {
    pub os: Option<String>,
    pub machine: Option<String>,
    pub kernel: Option<String>,
    pub uptime: Option<String>,
    pub username: Option<Arc<str>>,
    pub hostname: Option<Arc<str>>,
    pub resolution: Vec<Arc<str>>,
    pub wm: Option<String>,
    pub de: Option<String>,
    pub shell: Option<String>,
    pub cpu: Option<String>,
    pub font: Option<String>,
    pub cursor: Option<String>,
    pub terminal: Option<String>,
    pub terminal_font: Option<String>,
    pub gpus: Vec<Arc<str>>,
    pub memory: Option<String>,
    pub disks: Vec<(String, String)>,
    pub battery: Option<String>,
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub icons: Option<String>,
    pub ip: Vec<Arc<str>>,
    pub id: Arc<str>,
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
        let mut id: Arc<str> = Arc::from("");
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
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn as_vec(self) -> Vec<(String, String)> {
        let username = self.username.unwrap_or_else(|| Arc::from(""));
        let hostname = self.hostname.unwrap_or_else(|| Arc::from(""));
        let y = format!("{username}@{hostname}");
        let repeats = y.len();
        let (dark, light) = palette();
        let mut res: Vec<(String, String)> = vec![
            (y, Some(String::new())),
            (["-"].repeat(repeats).join(""), Some(String::new())),
            // if none, empty string
            // if not none
            ("OS".to_string(), self.os),
            ("Host".to_string(), self.machine),
            ("Kernel".to_string(), self.kernel),
            ("Uptime".to_string(), self.uptime),
            ("Shell".to_string(), self.shell),
            ("WM".to_string(), self.wm),
            ("DE".to_string(), self.de),
            ("CPU".to_string(), self.cpu),
            ("Theme".to_string(), self.theme),
            ("System Font".to_string(), self.font),
            ("Cursor".to_string(), self.cursor),
            ("Terminal".to_string(), self.terminal),
            ("Terminal Font".to_string(), self.terminal_font),
            ("Memory".to_string(), self.memory),
            ("Battery".to_string(), self.battery),
            ("Locale".to_string(), self.locale),
            ("Icon Theme".to_string(), self.icons),
        ]
        .into_iter()
        .filter_map(|(x, y)| y.map(|z| (x, z)))
        .chain(
            self.resolution
                .into_iter()
                .enumerate()
                .map(|(idx, res)| (format!("Display {}", idx + 1), res.to_string())),
        )
        .chain(
            self.gpus
                .into_iter()
                .enumerate()
                .map(|(idx, res)| (format!("GPU {}", idx + 1), res.to_string())),
        )
        .chain(self.disks)
        .chain(
            self.ip
                .into_iter()
                .map(|x| ("IP".to_string(), x.to_string())),
        )
        .collect();
        res.push((String::new(), dark));
        res.push((String::new(), light));
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
        val: Option<&String>,
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

fn palette() -> (String, String) {
    (
        (0..8u8)
            .map(|x| "   ".on(Color::AnsiValue(x)).to_string())
            .collect::<String>(),
        (8..16u8)
            .map(|x| "   ".on(Color::AnsiValue(x)).to_string())
            .collect::<String>(),
    )
}
