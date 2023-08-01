use std::{collections::HashMap, env, fs, iter::zip, path::Path};

use directories::ProjectDirs;
use regex::Regex;
use rkyv::{check_archived_root, Archive};
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

type UnprocessedAsciiArtVec = Vec<AsciiArtUnprocessed>;

#[derive(serde::Serialize, serde::Deserialize, rkyv::Serialize, Archive, Debug)]
#[archive(check_bytes)]
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
    let flags: HashMap<String, Vec<(u8, u8, u8)>> = serde_yaml //todo: switch to css hex strings
        ::from_str(binding.as_str())
    .unwrap();
    let flags_archived = rkyv::to_bytes::<_, 1024>(&flags).unwrap();
    check_archived_root::<HashMap<String, Vec<(u8, u8, u8)>>>(&flags_archived).unwrap();
    // let flags_archived = bson::to_vec( bson::doc!{"flags"&flags).unwrap();
    let out_dir = (env::var("OUT_DIR").unwrap() + "/../../../data").into_boxed_str(); //todo: see if there's a less hacky way to do this
                                                                                      // println!("cargo:warning={out_dir}");
    fs::DirBuilder::new().create(out_dir.as_ref()).ok();

    // Archive Icons

    // Read from json
    let icons = load_icons_from_yaml(Path::new("data/data.yaml"));
    //save archived icons
    let icons_archived = rkyv::to_bytes::<_, 1024>(&icons).unwrap();
    check_archived_root::<Vec<AsciiArt>>(&icons_archived).unwrap();
    // let icons_archived = bson::to_vec(&icons).unwrap();
    match std::env::var("PROFILE").unwrap().as_str() {
        "debug" => {
            println!("cargo:warning=File len{:?}", icons_archived.len());
            fs::write(out_dir.to_string() + "/icons.rkyv", &icons_archived).unwrap();
            fs::write(out_dir.to_string() + "/flags.rkyv", &flags_archived).unwrap();
        }

        "release" => {
            fs::DirBuilder::new().create(proj_dirs.data_dir()).ok();
            fs::copy("data/flags.json", proj_dirs.data_dir().join("/icons.yaml")).unwrap();
            fs::write(proj_dirs.data_dir().join("/icons.rkyv"), &icons_archived).unwrap();
            fs::copy("data/flags.json", proj_dirs.data_dir().join("/flags.json")).unwrap();
            fs::write(proj_dirs.data_dir().join("/flags.rkyv"), &flags_archived).unwrap();
        }
        _ => panic!("Unknown profile"),
    }
}

fn load_icons_from_yaml(path: &Path) -> Vec<AsciiArt> {
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
