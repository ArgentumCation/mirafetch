use core::iter::zip;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use rkyv::Archive;

type UnprocessedAsciiArtVec = Vec<AsciiArtUnprocessed>;
pub const ICON_FILE: &str = "icons.yaml";
pub const FLAG_FILE: &str = "flags.toml";

#[derive(serde::Serialize, serde::Deserialize, Archive, Debug, Clone, rkyv::Serialize)]
#[archive(check_bytes)]
pub enum Color {
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AsciiArtUnprocessed {
    name: Vec<String>,
    colors: Vec<Color>,
    width: u16,
    art: String,
}

#[derive(serde::Serialize, serde::Deserialize, rkyv::Serialize, Archive, Debug)]
#[archive(check_bytes)]
struct AsciiArt {
    names: Vec<String>,
    colors: Vec<Color>,
    width: u16,
    text: Vec<(u8, String)>,
}
pub fn archive_icons(out_file: &Path) -> anyhow::Result<()> {
    let icons_resources_path = &get_resource_dir().join(ICON_FILE);
    let icons_local_path = &get_executable_dir()?.join(ICON_FILE);

    // if icons.yaml doesn't exist or is older than the local copy, copy our version to the resources directory
    if !icons_resources_path.exists()
        || icons_resources_path.metadata()?.modified()?
            < fs::metadata(icons_local_path)?.modified()?
    {
        fs::copy(icons_local_path, icons_resources_path)?;
    }
    let icons = load_icons_from_yaml(icons_resources_path);
    //save archived icons
    let icons_archived = rkyv::to_bytes::<_, 1024>(&icons)?;
    fs::write(out_file, &icons_archived)?;
    Ok(())
}

fn get_executable_dir() -> anyhow::Result<PathBuf> {
    #[cfg(debug_assertions)]
    return Ok(Path::new(&env::var("OUT_DIR").unwrap()).join("../../../")); //todo: see if there's a less hacky way to do this

    #[cfg(not(debug_assertions))]
    Ok(env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not get directory of current executable"))?
        .to_owned())
}

pub fn archive_flags(out_file: &Path) -> anyhow::Result<()> {
    let flags_resources_path = &get_resource_dir().join(FLAG_FILE);
    let flags_local_path = &get_executable_dir()?.join(FLAG_FILE);
    eprintln!("{:?},{:?}", flags_resources_path, flags_local_path);
    // if flags.toml doesn't exist or is older than the local copy, copy our version to the resources directory
    if !flags_resources_path.exists()
        || flags_resources_path.metadata()?.modified()?
            < fs::metadata(flags_local_path)?.modified()?
    {
        fs::copy(flags_local_path, flags_resources_path)?;
    }
    let flags_file = fs::read_to_string(flags_resources_path).unwrap();
    let flags: HashMap<String, Vec<(u8, u8, u8)>> = toml //todo: switch to css hex strings
        ::from_str(flags_file.as_str())
    .unwrap();
    let flags_archived = rkyv::to_bytes::<_, 1024>(&flags).unwrap();
    fs::write(out_file, &flags_archived).unwrap();
    Ok(())
}
fn load_icons_from_yaml(path: &Path) -> Vec<AsciiArt> {
    eprintln!("{}", path.to_str().unwrap());
    let files = fs::read_dir(std::env::current_dir().unwrap().join("data"))
        .unwrap()
        .map(|x| x.unwrap().path().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();
    eprintln!("{:?}", files);
    let binding = fs::read_to_string(path).unwrap();
    let data_yaml: UnprocessedAsciiArtVec = serde_yaml::from_str(&binding).unwrap();
    process_ascii_art(data_yaml)
}
fn process_ascii_art(data: Vec<AsciiArtUnprocessed>) -> Vec<AsciiArt> {
    let icons = {
        let regex = Regex::new(r#"\$\{c(\d*)\}"#).unwrap();
        data.iter()
            .map(|item| {
                // println!("cargo:warning={:?}: {:?}", item.name, item.colors);
                let color_idx = regex
                    .captures_iter(&item.art)
                    .map(|x| str::parse(x.get(1).unwrap().as_str()).unwrap())
                    .collect::<Vec<u8>>();
                let chunks = regex
                    .split(&item.art)
                    .map(|c| c.to_owned())
                    .skip(1)
                    .collect::<Vec<String>>();
                let ascii_art = (zip(color_idx, chunks)).collect();
                AsciiArt {
                    names: item
                        .name
                        .clone()
                        .into_iter()
                        .map(|x| x.to_lowercase())
                        .collect(),
                    colors: item.colors.to_vec(),
                    width: item.width,
                    text: ascii_art,
                }
            })
            .collect::<Vec<AsciiArt>>()
    };
    icons
}

pub fn get_resource_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    return (Path::new(&env::var("OUT_DIR").unwrap()).join("../../../")).to_owned(); //todo: see if there's a less hacky way to do this

    #[cfg(not(debug_assertions))]
    return ProjectDirs::from("", "", "Mirafetch")
        .ok_or_else(|| {
            anyhow!(
                "Could not find a project directory for Mirafetch. Please report this as a bug."
            )
        })
        .unwrap()
        .data_dir()
        .to_owned();
}
