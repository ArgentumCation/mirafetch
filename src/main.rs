#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{ Result };
use crossterm::{
    cursor::{ MoveTo, MoveToColumn, MoveToNextLine },
    style::{ PrintStyledContent, Stylize },
    terminal::{ Clear, ClearType::All },
    ExecutableCommand,
};
use mirafetch::{
    util::{ AsciiArt, get_icon, get_colorscheme },
    Colorizer,
    Config,
    Info,
    Orientation,
};

// mod linuxinfo;

// use rkyv::{ validation::validators::check_archived_root, Deserialize, Infallible };

use std::{ io::stdout };
mod util;

fn main() -> anyhow::Result<()> {
    // Load Settings
    // todo: load from TOML
    let settings = Config::new(
        "transgender".to_string(),
        Orientation::Horizontal,
        true,
        "Windows".to_string()
    );

    // Get Color Scheme from archive
    let scheme = get_colorscheme(&settings.scheme_name).unwrap();
    let logo: AsciiArt = get_icon(&settings.icon_name).unwrap();
    let colorizer = Colorizer { color_scheme: scheme, orientation: settings.orientation };

    // Get system info
    let info = Info::new().as_vec();

    // Colorize
    let icon = if settings.gay {
        colorizer.gay_colorize(&logo)
    } else {
        colorizer.straight_colorize(&logo)
    };

    display(icon, info, logo)?;
    Ok(())
}

fn display(
    mut icon: Vec<crossterm::style::StyledContent<String>>,
    mut info: Vec<(String, String)>,
    logo: AsciiArt
) -> Result<(), anyhow::Error> {
    icon.reverse();
    info.reverse();
    stdout().execute(Clear(All))?.execute(MoveTo(0, 0))?;
    while let Some(line) = icon.pop() {
        stdout() /* .execute(ResetColor)?*/
            .execute(PrintStyledContent(line))?;
    }
    stdout().execute(MoveTo(0, 0))?;
    Ok(while let Some(line) = info.pop() {
        let (x, y) = line;
        stdout()
            .execute(MoveToColumn(logo.width + 3))?
            .execute(PrintStyledContent(x.clone().bold().red()))?;
        if !x.is_empty() && !y.is_empty() {
            stdout().execute(PrintStyledContent(": ".bold().red()))?;
        }
        stdout().execute(PrintStyledContent(y.reset()))?.execute(MoveToNextLine(1))?;
    })
}
