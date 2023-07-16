use std::{collections::HashMap, env, fs, iter::zip};

use regex::{self, Regex};
use rkyv::{self, Archive};

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
    //this should work with yaml serde without changes
    println!("cargo:rerun-if-changed=data/flags.json");
    //todo: make this yaml
    println!("cargo:rerun-if-changed=data/data.json5");

    //Archive Flags
    let binding = fs::read_to_string("data/flags.json").unwrap();
    let flags_json: HashMap<String, Vec<(u8, u8, u8)>> = json5 //todo: switch to css hex strings
        ::from_str(binding.as_str())
    .unwrap();
    let bytes = rkyv::to_bytes::<_, 1024>(&flags_json).unwrap();
    let out_dir = (env::var("OUT_DIR").unwrap() + "/../../../data").into_boxed_str(); //todo: see if there's a less hacky way to do this
                                                                                      // println!("cargo:warning={out_dir}");
    fs::DirBuilder::new().create(out_dir.as_ref()).ok();
    fs::write(out_dir.to_string() + "/flags.rkyv", bytes).unwrap();

    // Archive Icons

    // Read from json
    let binding = fs::read_to_string("data/data.json5").unwrap();
    let data_json: UnprocessedAsciiArtVec = json5::from_str(&binding).unwrap();
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
            let ascii_art = Vec::from_iter(zip(color_idx, chunks));
            AsciiArt {
                names: item.name.to_owned(),
                colors: item.colors.to_vec(),
                width: item.width,
                text: ascii_art,
            }
        })
        .collect::<Vec<AsciiArt>>();

    //save archived icons
    let bytes = rkyv::to_bytes::<_, 1024>(&icons_json).unwrap();
    fs::write(out_dir.to_string() + "/icons.rkyv", bytes).unwrap();
}
