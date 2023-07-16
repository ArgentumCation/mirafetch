use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use anyhow::anyhow;
use crossterm::style::Color;
use rkyv::Infallible;
use rkyv::archived_root;
use rkyv::with::ArchiveWith;
use rkyv::with::DeserializeWith;
use rkyv::with::Map;
use rkyv::{ Archive, Deserialize };
use rkyv_with::{ ArchiveWith, DeserializeWith };
use sysinfo::{ get_current_pid, CpuExt, ProcessExt, System, SystemExt, UserExt };
use std::path::Path;

pub trait OSInfo {
    fn new() -> Self;
    fn displays(&self) -> Vec<String> {
        todo!();
    }

    fn machine(&self) -> Option<String> {
        todo!();
    }

    fn kernel(&self, s: &System) -> Option<String> {
        s.kernel_version()
    }

    fn gpus(&self) -> Option<Vec<String>> {
        todo!();
    }

    fn theme(&self) -> Option<String> {
        todo!();
    }
    fn wm(&self) -> Option<String> {
        todo!();
    }

    fn de(&self) -> Option<String> {
        todo!();
    }

    fn shell(&self, s: &System) -> Option<String> {
        let pid = get_current_pid().ok()?;
        let parent_pid = s.process(pid)?.parent()?;
        let parent = s.process(parent_pid)?.name();
        Some(parent.replace(".exe", ""))
    }
    #[allow(clippy::cast_precision_loss)]
    fn cpu(&self, sys: &System) -> Option<String> {
        let cpu = &sys.cpus().get(0)?;
        Some(
            sys.physical_core_count().map_or_else(
                || format!("{} @ {}MHz", cpu.brand(), cpu.frequency()),
                |cores| {
                    format!(
                        "{} ({}) @ {}GHz",
                        cpu.brand(),
                        cores,
                        (cpu.frequency() as f32) / 1000.0
                    )
                }
            )
        )
    }

    fn username(&self, sys: &System) -> Option<String> {
        Some(
            sys.get_user_by_id(sys.process(get_current_pid().ok()?)?.user_id()?)?.name().to_string()
        )
    }
}
pub fn get_icon(icon_name: &str) -> anyhow::Result<AsciiArt> {
    let path = std::env::current_exe()?.parent().unwrap().join(Path::new("data/icons.rkyv"));
    // println!("{path:#?}");
    let binding = fs::read(path)?;
    let archived = unsafe { archived_root::<Vec<AsciiArtRemote>>(&binding) };
    let icons: Vec<AsciiArtRemote> = archived.deserialize(&mut Infallible).unwrap();
    icons
        .into_iter()
        .find(|item| item.names.contains(&icon_name.to_string()))
        .map(std::convert::Into::into)
        .ok_or_else(|| anyhow!(""))
}

pub fn get_colorscheme(scheme_name: &str) -> anyhow::Result<Arc<[Color]>> {
    let path = std::env::current_exe()?.parent().unwrap().join(Path::new("data/flags.rkyv"));
    println!("{path:#?}");
    let binding = fs::read(path)?;
    let schemes: HashMap<String, Vec<(u8, u8, u8)>> = (
        unsafe {
            archived_root::<HashMap<String, Vec<(u8, u8, u8)>>>(binding.as_slice())
        }
    ).deserialize(&mut Infallible)?;

    Ok(
        schemes[scheme_name]
            .clone()
            .into_iter()
            .map(|(r, g, b)| Color::Rgb { r, g, b })
            .collect()
    )
}
#[derive(
    serde::Serialize,
    serde::Deserialize,
    Archive,
    ArchiveWith,
    Debug,
    DeserializeWith,
    rkyv::Deserialize,
    Clone
)]
#[archive_with(from(Color))]
enum ColorRemote {
    /// Resets the terminal color.
    Reset,

    /// Black color.
    Black,

    /// Dark grey color.
    DarkGrey,

    /// Light red color.
    Red,

    /// Dark red color.
    DarkRed,

    /// Light green color.
    Green,

    /// Dark green color.
    DarkGreen,

    /// Light yellow color.
    Yellow,

    /// Dark yellow color.
    DarkYellow,

    /// Light blue color.
    Blue,

    /// Dark blue color.
    DarkBlue,

    /// Light magenta color.
    Magenta,

    /// Dark magenta color.
    DarkMagenta,

    /// Light cyan color.
    Cyan,

    /// Dark cyan color.
    DarkCyan,

    /// White color.
    White,

    /// Grey color.
    Grey,

    /// An RGB color. See [RGB color model](https://en.wikipedia.org/wiki/RGB_color_model) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },

    /// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    AnsiValue(u8),
}
pub struct AsciiArt {
    pub names: Vec<String>,
    pub colors: Vec<Color>,
    pub width: u16,
    pub text: Vec<(u8, String)>,
}
#[derive(
    serde::Serialize,
    serde::Deserialize,
    rkyv::Serialize,
    Archive,
    Debug,
    ArchiveWith,
    Deserialize,
    Clone
)]
#[archive_with(from(AsciiArt))]
struct AsciiArtRemote {
    pub names: Vec<String>,
    #[archive_with(from(Vec<Color>), via(Map<ColorRemote>))]
    pub colors: Vec<ColorRemote>,
    pub width: u16,
    pub text: Vec<(u8, String)>,
}
// impl DeserializeUnsized<[[AsciiArtBak]], Infallible> for [AsciiArtBak] {}
impl From<AsciiArtRemote> for AsciiArt {
    fn from(other: AsciiArtRemote) -> Self {
        Self {
            names: other.names,
            colors: other.colors.into_iter().map(std::convert::Into::into).collect(),
            width: other.width,
            text: other.text,
        }
    }
}
impl From<ColorRemote> for Color {
    fn from(val: ColorRemote) -> Self {
        match val {
            ColorRemote::Reset => Self::Reset,
            ColorRemote::Black => Self::Black,
            ColorRemote::DarkGrey => Self::DarkGrey,
            ColorRemote::Red => Self::Red,
            ColorRemote::DarkRed => Self::DarkRed,
            ColorRemote::Green => Self::Green,
            ColorRemote::DarkGreen => Self::DarkGreen,
            ColorRemote::Yellow => Self::Yellow,
            ColorRemote::DarkYellow => Self::DarkYellow,
            ColorRemote::Blue => Self::Blue,
            ColorRemote::DarkBlue => Self::DarkBlue,
            ColorRemote::Magenta => Self::Magenta,
            ColorRemote::DarkMagenta => Self::DarkMagenta,
            ColorRemote::Cyan => Self::Cyan,
            ColorRemote::DarkCyan => Self::DarkCyan,
            ColorRemote::White => Self::White,
            ColorRemote::Grey => Self::Grey,
            ColorRemote::Rgb { r, g, b } => Self::Rgb { r, g, b },
            ColorRemote::AnsiValue(x) => Self::AnsiValue(x),
        }
    }
}
