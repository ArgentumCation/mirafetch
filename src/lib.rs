#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![warn(clippy::style)]
#![allow(clippy::cast_precision_loss)]
use crate::util::AsciiArt;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
// use anyhow::Ok;
#[cfg(target_family = "windows")]
use crate::wininfo::WindowsInfo as get_info;
use crossterm::style::{Color, StyledContent, Stylize};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use std::fmt::Display;

use std::sync::Arc;

use util::OSInfo;
mod wininfo;
#[cfg(target_family = "unix")]
use crate::linuxinfo::LinuxInfo as get_info;
mod linuxinfo;
mod tests;
pub mod util;
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub scheme_name: Option<Box<str>>,
    pub orientation: Option<Orientation>,
    pub icon_name: Option<Box<str>>,
}

pub trait Colorizer {
    /// .
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>>;
}

pub struct FlagColorizer {
    pub color_scheme: Arc<[Color]>,
    pub orientation: Orientation,
}
impl Config {
    /// Builder method to add an icon to config
    #[must_use]
    pub fn with_icon(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(Into::<String>::into(icon_name).into_boxed_str());
        self
    }
    /// Create new struct containing user settings
    #[must_use]
    pub fn new(
        scheme_name: Option<impl ToString>,
        orientation: Option<Orientation>,
        icon_name: Option<impl ToString>,
    ) -> Self {
        Self {
            scheme_name: scheme_name.map(|x| x.to_string().into_boxed_str()),
            orientation,
            icon_name: icon_name.map(|x| x.to_string().into_boxed_str()),
        }
    }
}

impl FlagColorizer {
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
impl Colorizer for FlagColorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let txt: String = ascii_art
            .text
            .clone()
            .into_par_iter()
            .map(|x| x.1)
            .collect();

        match self.orientation {
            Orientation::Horizontal => {
                let colors = self.length_to_colors(txt.par_lines().count());

                txt.par_lines()
                    .collect::<Vec<&str>>()
                    .par_iter()
                    .enumerate()
                    .map(move |(i, l)| ((*l).to_string() + "\n").with(colors[i]))
                    .collect::<Vec<_>>()
            }

            Orientation::Vertical => {
                //Requires txt has at least one line and is rectangular
                let colors = self.length_to_colors(ascii_art.width as usize);

                txt.par_lines()
                    .flat_map(|line| {
                        line.par_char_indices()
                            .map(|(idx, ch)| ch.to_string().with(colors[idx]))
                            .chain([String::from("\n").with(Color::Reset)])
                    })
                    .collect()
            }
        }
    }
}

pub struct DefaultColorizer {}

impl Colorizer for DefaultColorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let colors = &ascii_art.colors;
        ascii_art
            .text
            .par_iter()
            .map(|(idx, text)| -> StyledContent<String> {
                text.clone().with(*colors.get((*idx as usize) - 1).unwrap())
            })
            .collect::<Vec<StyledContent<String>>>()
    }
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct Info {
    // general_readout: Rc<GeneralReadout>,
    os: Option<String>,
    machine: Option<String>,
    kernel: Option<String>,
    uptime: Option<String>,
    username: Option<Arc<str>>,
    hostname: Option<Arc<str>>,
    resolution: Vec<Arc<str>>,
    wm: Option<String>,
    de: Option<String>,
    shell: Option<String>,
    cpu: Option<String>,
    font: Option<String>,
    cursor: Option<String>,
    terminal: Option<String>,
    terminal_font: Option<String>,
    gpus: Vec<Arc<str>>,
    memory: Option<String>,
    disks: Vec<(String, String)>,
    battery: Option<String>,
    locale: Option<String>,
    theme: Option<String>,
    icons: Option<String>,
    ip: Vec<Arc<str>>,
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
    fn new() -> Self {
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
