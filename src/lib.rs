#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![warn(clippy::style)]
#![allow(clippy::cast_precision_loss)]
// use regex::Replacer;
use crate::util::AsciiArt;
use rayon::iter::{ ParallelIterator, IndexedParallelIterator };
// use anyhow::Ok;
use rayon::prelude::*;
use serde::{ Serialize, Deserialize };
use util::OSInfo;
use std::{ sync::Arc };
use std::fmt::Display;
use sysinfo::{ System, SystemExt };
use time::Duration;
use crossterm::{ style::{ Color, Stylize, StyledContent } };
#[cfg(target_family = "windows")]
use crate::wininfo::WindowsInfo as get_info;
mod wininfo;
#[cfg(target_family = "unix")]
use crate::linuxinfo::UnixInfo as get_info;
mod linuxinfo;
pub mod util;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub scheme_name: Option<Box<str>>,
    pub orientation: Option<Orientation>,
    pub gay: bool,
    pub icon_name: Box<str>,
}

pub trait Colorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>>;
}

pub struct GayColorizer {
    pub color_scheme: Arc<[Color]>,
    pub orientation: Orientation,
}
impl Config {
    #[must_use]
    pub fn new(
        scheme_name: Option<String>,
        orientation: Option<Orientation>,
        gay: Option<bool>,
        icon_name: Option<String>
    ) -> Self {
        Self {
            scheme_name: match scheme_name {
                Some(x) => Some(x.into_boxed_str()),
                None => Some("transgender".to_string().into_boxed_str()), //todo!(),
            },
            orientation,
            gay: gay.unwrap_or_default(),
            icon_name: match icon_name {
                Some(x) => x.into_boxed_str(),
                None => "Arch".to_string().into_boxed_str(), //todo!(),
            },
        }
    }
}
impl GayColorizer {
    fn length_to_colors(&self, length: usize) -> Vec<Color> {
        let preset_len = self.color_scheme.len(); //6
        let center = preset_len / 2; // 4

        let repeats = length / preset_len; // 1
        let mut weights = [repeats].repeat(preset_len);
        let mut extras = length % preset_len; // 2
        if extras % 2 == 1 {
            extras -= 1;
            weights[center] += 1;
        }
        let mut border = 0;
        while extras > 0 {
            extras -= 2; //0
            weights[border] += 1; //
            weights[preset_len - border - 1] += 1;
            border += 1;
        }
        self.weights_to_colors(weights)
    }
    fn weights_to_colors(&self, weights: Vec<usize>) -> Vec<Color> {
        weights
            .into_par_iter()
            .enumerate()
            .flat_map(|(idx, weight)| {
                let mut v: Vec<Color> = [self.color_scheme[idx]].repeat(weight);
                v.fill(self.color_scheme[idx]);
                v
            })
            .collect::<Vec<Color>>()
    }
}
impl Colorizer for GayColorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let txt: String = ascii_art.text
            .clone()
            .into_par_iter()
            .map(|x| x.1)
            .collect();
        match self.orientation {
            Orientation::Horizontal => {
                let colors = self.length_to_colors(txt.par_lines().count());

                txt.lines()
                    .enumerate()
                    .par_bridge()
                    .map(move |(i, l)| { (l.to_string() + "\n").with(colors[i]) })
                    .collect::<Vec<_>>()
            }

            Orientation::Vertical => {
                //Requires txt has at least one line and is rectangular
                let colors = self.length_to_colors(ascii_art.width as usize);

                txt.par_lines()
                    .flat_map(|line| {
                        line.par_char_indices()
                            .map(|(idx, ch)| { ch.to_string().with(colors[idx]) })
                            .chain([String::from("\n").with(Color::Reset)])
                    })
                    .collect()
            }
        }
        // Vertical
    }
}

pub struct DefaultColorizer {}

impl Colorizer for DefaultColorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let colors = &ascii_art.colors;
        ascii_art.text
            .par_iter()
            .map(
                |(idx, text)| -> StyledContent<String> {
                    text.clone().with(*colors.get((*idx as usize) - 1).unwrap())
                }
            )
            .collect::<Vec<StyledContent<String>>>()
    }
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Default)]
pub struct Info {
    // general_readout: Rc<GeneralReadout>,
    os: Option<String>,
    machine: Option<String>,
    kernel: Option<String>,
    uptime: Option<String>,
    username: Option<String>,
    hostname: Option<String>,
    resolution: Vec<String>,
    wm: Option<String>,
    de: Option<String>,
    shell: Option<String>,
    cpu: Option<String>,
    font: Option<String>,
    cursor: Option<String>,
    terminal: Option<String>,
    terminal_font: Option<String>,
    gpus: Vec<String>,
    memory: Option<String>,
    disks: Option<String>,
    battery: Option<String>,
    locale: Option<String>,
    theme: Option<String>,
    icons: Option<String>,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // println!("{:#?}", self);
        let username = self.username.clone().unwrap_or_default();
        let hostname = self.hostname.clone().unwrap_or_default();
        let y = format!("{username}@{hostname}");
        let repeats = y.len();
        write!(f, "{}{}", y + "\n", ["-"].repeat(repeats).join("") + "\n")?;
        // if none, empty string
        // if not none
        info_fmt(f, "OS", self.os.as_ref())?;
        info_fmt(f, "Host", self.machine.as_ref())?;
        info_fmt(f, "Kernel", self.kernel.as_ref())?;
        info_fmt(f, "Uptime", self.uptime.as_ref())?;
        info_fmt(f, "Shell", self.shell.as_ref())?;
        for (idx, res) in self.resolution.iter().enumerate() {
            info_fmt(f, format!("Display {}", idx + 1).as_str(), Some(res))?;
        }
        for (idx, res) in self.gpus.iter().enumerate() {
            info_fmt(f, format!("GPU {}", idx + 1).as_str(), Some(res))?;
        }
        info_fmt(f, "WM", self.wm.as_ref())?;
        info_fmt(f, "DE", self.de.as_ref())?;
        info_fmt(f, "CPU", self.cpu.as_ref())?;
        info_fmt(f, "Theme", self.theme.as_ref())?;
        info_fmt(f, "System Font", self.font.as_ref())?;
        info_fmt(f, "Cursor", self.cursor.as_ref())?;
        info_fmt(f, "Terminal", self.terminal.as_ref())?;
        info_fmt(f, "Terminal Font", self.terminal_font.as_ref())?;
        info_fmt(f, "Memory", self.memory.as_ref())?;
        info_fmt(f, "Disks", self.disks.as_ref())?;
        info_fmt(f, "Battery", self.battery.as_ref())?;
        info_fmt(f, "Locale", self.locale.as_ref())?;
        info_fmt(f, "Icon Theme", self.icons.as_ref())?;
        palette();
        Ok::<_, std::fmt::Error>(())
    }
}

fn palette() -> (String, String) {
    (
        (0..8u8).map(|x| "   ".on(Color::AnsiValue(x)).to_string()).collect::<String>(),
        (8..16u8).map(|x| "   ".on(Color::AnsiValue(x)).to_string()).collect::<String>(),
    )
}

#[cfg(target_family = "unix")]
fn get_kernel() -> Option<String> {
    use sysinfo::RefreshKind;

    let sys = System::new_all();
    return sys.kernel_version();
}

impl Info {
    #[must_use]
    pub fn new() -> Self {
        let mut sys = System::new_all();
        let getter = get_info::new();
        sys.refresh_all();
        Self {
            // general_readout: general_readout.clone(),
            os: sys.long_os_version(),
            machine: getter.machine(),
            kernel: getter.kernel(&sys),
            uptime: Some(Duration::new(sys.uptime().try_into().unwrap(), 0).to_string()),
            username: getter.username(&sys),
            hostname: sys.host_name(),
            resolution: getter.displays(),
            wm: getter.wm(),
            de: getter.de(),
            shell: getter.shell(&sys),
            cpu: getter.cpu(&sys),
            font: None, //todo!(),
            cursor: None, //todo!(),
            terminal: None, //todo!(),
            terminal_font: None, //todo!(),
            gpus: getter.gpus().unwrap_or_default(),
            memory: None, //todo!(),
            disks: None, //todo!(),
            battery: None, //todo!(),
            locale: None, //todo!(),
            theme: getter.theme(), //todo!(),
            icons: None, //todo!(),
        }
    }
    #[must_use]
    pub fn as_vec(self) -> Vec<(String, String)> {
        let username = self.username.clone().unwrap_or_default();
        let hostname = self.hostname.clone().unwrap_or_default();
        let y = format!("{username}@{hostname}");
        let repeats = y.len();
        let (dark, light) = palette();
        let mut res = vec![
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
            ("Disks".to_string(), self.disks),
            ("Battery".to_string(), self.battery),
            ("Locale".to_string(), self.locale),
            ("Icon Theme".to_string(), self.icons)
        ]
            .into_iter()
            .filter_map(|(x, y)| y.map(|z| (x, z)))
            .chain(
                self.resolution
                    .into_iter()
                    .enumerate()
                    .map(|(idx, res)| { (format!("Display {}", idx + 1), res) })
            )
            .chain(
                self.gpus
                    .into_iter()
                    .enumerate()
                    .map(|(idx, res)| { (format!("GPU {}", idx + 1), res) })
            )
            .collect::<Vec<(String, String)>>();
        res.push((String::new(), dark));
        res.push((String::new(), light));
        res
    }
}

fn info_fmt(
    f: &mut std::fmt::Formatter<'_>,
    info_type: &str,
    val: Option<&String>
) -> std::fmt::Result {
    if let Some(x) = val {
        info_type.with(Color::Red).attribute(crossterm::style::Attribute::Bold).fmt(f)?;
        x.clone().reset().fmt(f)?;
        '\n'.fmt(f)?;
    }
    Ok(())
}
