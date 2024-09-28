use anyhow::anyhow;
use crossterm::style::Color;
use num::Unsigned;
use regex::Regex;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeAs};
use std::{iter::zip, num::ParseIntError, str::FromStr, sync::Arc};

const ICON_FILE: &str = include_str!("../data/icons.yaml");
const FLAGS_FILE: &str = include_str!("../data/flags.toml");
/// .
///
/// # Errors
///
/// This function will return an error if the icon cannot be found
#[allow(dead_code)]
pub fn get_icon<'a>(icon_name: impl ToString) -> anyhow::Result<AsciiArt> {
    let icon_name = &icon_name.to_string().to_ascii_lowercase();
    let icons = serde_yaml::from_str::<Vec<AsciiArtUnprocessed>>(ICON_FILE)
        .expect("Could not parse icons file")
        .into_iter()
        .map(|x| TryInto::<AsciiArt>::try_into(x).expect("Could not parse icon"));
    icons
        .into_iter()
        .find(|item| item.name.contains(&icon_name.to_string()))
        .map(std::convert::Into::into)
        .ok_or_else(|| anyhow!(format!("Could not find an icon for {icon_name}")))
}

/// TODO
///
/// # Errors
///
/// This function will return an error if the colorscheme cannot be found
#[allow(dead_code)]
pub fn get_colorscheme<'a>(scheme_name: &impl ToString) -> Arc<[Color]> {
    let scheme = scheme_name.to_string();
    let schemes: FxHashMap<String, Vec<(u8, u8, u8)>> =
        toml::from_str(FLAGS_FILE).expect("Failed to parse flags.toml");
    schemes
        .get(&scheme)
        .unwrap_or_else(|| panic!("Failed to find scheme {}", &scheme))
        .iter()
        .map(|(r, g, b)| Color::Rgb {
            r: *r,
            g: *g,
            b: *b,
        })
        .collect()
}
pub struct AsciiArt {
    pub name: Vec<String>,
    pub colors: Vec<Color>,
    pub width: u16,
    pub height: u16,
    pub art: Vec<(u8, String)>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AsciiArtUnprocessed {
    pub name: Vec<String>,
    #[serde_as(as = "Vec<ColorRemote>")]
    pub colors: Vec<Color>,
    pub width: u16,
    pub art: String,
}
impl TryFrom<AsciiArtUnprocessed> for AsciiArt {
    fn try_from(val: AsciiArtUnprocessed) -> anyhow::Result<Self> {
        let regex = Regex::new(r"\$\{c(\d*)\}")?;
        let height = val.art.lines().count() as u16;
        let color_idx: Vec<u8> = regex
            .captures_iter(&val.art)
            .map(|x| -> anyhow::Result<u8> {
                str::parse(
                    x.get(1)
                        .ok_or_else(|| anyhow!("Invalid Ascii Art"))?
                        .as_str(),
                )
                .map_err(|op: ParseIntError| anyhow!(op))
            })
            .map(std::result::Result::unwrap)
            .collect();
        let chunks = regex
            .split(&val.art)
            .map(std::borrow::ToOwned::to_owned)
            .skip(1)
            .collect::<Vec<String>>();
        let ascii_art = (zip(color_idx, chunks)).collect();
        Ok(Self {
            name: val
                .name
                .clone()
                .into_iter()
                .map(|x| x.to_lowercase())
                .collect(),
            colors: val.colors.clone(),
            width: val.width,
            height,
            art: ascii_art,
        })
    }

    type Error = anyhow::Error;
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
    for val in [0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8] {
        if (i >> (10 * (val + 1))) == 0.into() {
            return format!(
                "{:.precision$} {}",
                if precision == 0 {
                    let tmp: T = i >> (10 * val);
                    f64::from_str(tmp.to_string().as_str())
                        .unwrap_or_else(|_| panic!("Could not parse {tmp} into f64"))
                } else {
                    f64::from_str(i.to_string().as_str())
                        .unwrap_or_else(|_| panic!("Could not parse {i} into f64"))
                        / f64::powi(1024_f64, i32::from(val))
                },
                units[val as usize]
            );
        }
    }
    panic!("bytes: {i}, precision: {precision}")
}

// TODO move all this stuff into a private module or something
#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
// #[serde(with = "Vec::<ColorRemote>")]
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
    AnsiValue(u8),
}

impl<'de, T> DeserializeAs<'de, T> for ColorRemote
where
    T: From<Color>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let color = Self::deserialize(deserializer)?;
        Ok(T::from(color))
    }
}
