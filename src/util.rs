use anyhow::anyhow;
use crossterm::style::Color;
use num::Unsigned;
use rkyv::check_archived_root;

use std::str::FromStr;

use directories::ProjectDirs;
use rkyv::with::ArchiveWith;

use rkyv::with::DeserializeWith;
use rkyv::with::Map;
use rkyv::Infallible;
use rkyv::{Archive, Deserialize};
use rkyv_with::{ArchiveWith, DeserializeWith};
use rustc_hash::FxHashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// .
///
/// # Errors
///
/// This function will return an error if the icon cannot be found
#[allow(dead_code)]
pub fn get_icon(icon_name: &str) -> anyhow::Result<AsciiArt> {
    let proj_dirs = ProjectDirs::from("", "", "Mirafetch").ok_or_else(|| {
        anyhow!("Could not find a project directory for Mirafetch. Please report this as a bug.")
    })?;
    let path = {
        if proj_dirs.data_dir().exists() {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            // Dev environments only
            // TODO: create data dir and add icons or throw an error or something
            std::env::current_exe().map(|x| x.parent().unwrap().join("data"))
        }
    }?
    .join(Path::new("icons.rkyv"));
    let icon_name = &icon_name.to_ascii_lowercase();
    let binding = fs::read(path)?;
    let archived = check_archived_root::<Vec<AsciiArtRemote>>(&binding).unwrap();
    let icons: Vec<AsciiArtRemote> = archived.deserialize(&mut Infallible)?;
    icons
        .into_iter()
        .find(|item| item.names.contains(&icon_name.to_string()))
        .map(std::convert::Into::into)
        .ok_or_else(|| anyhow!(format!("Could not find an icon for {icon_name}")))
}

/// TODO
///
/// # Errors
///
/// This function will return an error if the colorscheme cannot be found
#[allow(dead_code)]
pub fn get_colorscheme(scheme_name: &str) -> anyhow::Result<Arc<[Color]>> {
    let proj_dirs = ProjectDirs::from("", "", "Mirafetch").unwrap();
    let path = {
        if proj_dirs.data_dir().exists() {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            std::env::current_exe().map(|x| x.parent().unwrap().join("data"))
        }
    }?
    .join(Path::new("flags.rkyv"));
    let binding = fs::read(path)?;
    let schemes: FxHashMap<String, Vec<(u8, u8, u8)>> =
        check_archived_root::<FxHashMap<String, Vec<(u8, u8, u8)>>>(&binding)?
            .deserialize(&mut Infallible)
            .unwrap();
    Ok(schemes[scheme_name]
        .iter()
        .map(|(r, g, b)| Color::Rgb {
            r: r.to_owned(),
            g: g.to_owned(),
            b: b.to_owned(),
        })
        .collect())
}
#[derive(
    serde::Serialize,
    serde::Deserialize,
    Archive,
    ArchiveWith,
    Debug,
    DeserializeWith,
    rkyv::Deserialize,
    Clone,
)]
#[archive_with(from(Color))]
#[archive(check_bytes)]
#[allow(clippy::used_underscore_binding)]
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
    Rgb { r: u8, g: u8, b: u8 },

    /// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    #[allow(clippy::used_underscore_binding)]
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
    Clone,
)]
#[archive_with(from(AsciiArt))]
#[archive(check_bytes)]
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
            colors: other
                .colors
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
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

#[allow(dead_code, clippy::cast_precision_loss)]
#[must_use]
pub fn bytecount_format<T>(i: T, precision: usize) -> String
where
    T: Unsigned
        + std::ops::Shr<u8, Output = T>
        + std::fmt::Display
        + PartialEq<T>
        + From<u8>
        + Copy,
{
    // let mut val = 0;
    let units = ["bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    // let i = f64::from_str(i.to_string().as_str()).unwrap();
    for val in [0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8] {
        if (i >> (10 * (val + 1))) == 0.into() {
            return format!(
                "{:.precision$} {}",
                if precision == 0 {
                    let tmp: T = i >> (10 * val);
                    f64::from_str(tmp.to_string().as_str()).unwrap()
                } else {
                    f64::from_str(i.to_string().as_str()).unwrap()
                        / f64::powi(1024_f64, i32::from(val))
                },
                units[val as usize]
            );
        }
    }
    panic!("bytes: {i}, precision: {precision}")
}
