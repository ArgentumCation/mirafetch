use std::{collections::HashMap, env, fs, iter::zip};

use directories::ProjectDirs;
use regex::Regex;
use rkyv::Archive;
#[derive(serde::Serialize, serde::Deserialize, Archive, Debug, Clone, rkyv::Serialize)]
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

type UnprocessedAsciiArtVec = Vec<AsciiArtUnprocessed>;

#[derive(serde::Serialize, serde::Deserialize, rkyv::Serialize, Archive, Debug)]
struct AsciiArt {
    names: Vec<String>,
    colors: Vec<Color>,
    width: u16,
    text: Vec<(u8, String)>,
}

fn main() {
    let proj_dirs = ProjectDirs::from("", "", "Mirafetch").unwrap();
    //this should work with yaml serde without changes
    println!("cargo:rerun-if-changed=data/flags.json");
    //todo: make this yaml
    // println!("cargo:rerun-if-changed=data/data2.json5");
    println!("cargo:rerun-if-changed=data/data.yaml");

    //Archive Flags
    let binding = fs::read_to_string("data/flags.json").unwrap();
    let flags_json: HashMap<String, Vec<(u8, u8, u8)>> = serde_yaml //todo: switch to css hex strings
        ::from_str(binding.as_str())
    .unwrap();
    let flag_bytes = rkyv::to_bytes::<_, 1024>(&flags_json).unwrap();
    let out_dir = (env::var("OUT_DIR").unwrap() + "/../../../data").into_boxed_str(); //todo: see if there's a less hacky way to do this
                                                                                      // println!("cargo:warning={out_dir}");
    fs::DirBuilder::new().create(out_dir.as_ref()).ok();

    // Archive Icons

    // Read from json
    let binding = fs::read_to_string("data/data.yaml").unwrap();
    let data_json: UnprocessedAsciiArtVec = serde_yaml::from_str(&binding).unwrap();
    // fs::write(
    //     "data/data.yaml",
    //     serde_yaml::to_string(&data_json).unwrap().as_bytes(),
    // )
    // .unwrap();
    // println!("cargo:warning={:#?}", data_json);
    let regex = Regex::new(r#"\$\{c(\d*)\}"#).unwrap();
    let icons_json = data_json
        .iter()
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
        .collect::<Vec<AsciiArt>>();

    //save archived icons
    let icon_bytes = rkyv::to_bytes::<_, 1024>(&icons_json).unwrap();
    match std::env::var("PROFILE").unwrap().as_str() {
        "debug" => {
            fs::write(out_dir.to_string() + "/icons.rkyv", &flag_bytes).unwrap();
            fs::write(out_dir.to_string() + "/flags.rkyv", &icon_bytes).unwrap();
        }
        "release" => {
            fs::copy("data/flags.json", proj_dirs.data_dir().join("/icons.yaml")).unwrap();
            fs::write(proj_dirs.data_dir().join("/icons.rkyv"), &icon_bytes).unwrap();
            fs::copy("data/flags.json", proj_dirs.data_dir().join("/flags.json")).unwrap();
            fs::write(proj_dirs.data_dir().join("/flags.rkyv"), &flag_bytes).unwrap();
        }
        _ => {}
    }
}
