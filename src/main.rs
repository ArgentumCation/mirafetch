#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{anyhow, Result};
use crossterm::{
    cursor::{position, MoveTo, MoveToColumn, MoveToNextLine},
    style::{Color, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType::All},
    ExecutableCommand,
};
use mirafetch::{
    util::{get_colorscheme, get_icon, AsciiArt},
    Colorizer, Config, DefaultColorizer, GayColorizer, Info,
};

// mod linuxinfo;

// use rkyv::{ validation::validators::check_archived_root, Deserialize, Infallible };

use std::{cmp::max, io::stdout, sync::Arc};
mod util;

fn main() -> anyhow::Result<()> {
    // Load Settings
    let info = Info::new();
    let id = info.id.clone();
    let info_vec = info.as_vec();
    // todo: load from TOML
    let settings = Config::new(
        None,
        None,
        None,
        Some(id.to_string()), //Todo: determine distro and change this to None
    );

    // Get Color Scheme from archive

    let scheme: Option<Arc<[Color]>> = settings
        .scheme_name
        .as_ref()
        .map_or_else(|| None, |x| get_colorscheme(x).ok());
    let logo: AsciiArt = get_icon(&settings.icon_name).unwrap();
    let colored_logo = colorize_logo(&settings, &scheme, &logo)?;

    // Get system info

    display(colored_logo, info_vec, &logo)?;
    Ok(())
}

fn colorize_logo(
    settings: &Config,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt,
) -> Result<Vec<crossterm::style::StyledContent<String>>, anyhow::Error> {
    let colorizer = if settings.gay {
        scheme.as_ref().map_or_else(
            || Err(anyhow!("Missing Scheme")),
            |scheme| {
                settings.orientation.map_or_else(
                    || Err(anyhow!("Missing Orientation")),
                    |orientation| {
                        Ok(Box::new(GayColorizer {
                            color_scheme: scheme.clone(),
                            orientation,
                        }) as Box<dyn Colorizer>)
                    },
                )
            },
        )
    } else {
        Ok(Box::new(DefaultColorizer {}) as Box<dyn Colorizer>)
    };
    Ok(colorizer?.colorize(logo))
}

fn display(
    mut icon: Vec<crossterm::style::StyledContent<String>>,
    mut info: Vec<(String, String)>,
    logo: &AsciiArt,
) -> Result<(), anyhow::Error> {
    icon.reverse();
    info.reverse();
    stdout().execute(Clear(All))?.execute(MoveTo(0, 0))?;
    while let Some(line) = icon.pop() {
        stdout() /* .execute(ResetColor)?*/
            .execute(PrintStyledContent(line))?;
    }
    let pos = position()?;
    stdout().execute(MoveTo(0, 0))?;
    while let Some(line) = info.pop() {
        let (x, y) = line;
        stdout()
            .execute(MoveToColumn(logo.width + 3))?
            .execute(PrintStyledContent(x.clone().bold().red()))?;
        if !x.is_empty() && !y.is_empty() {
            stdout().execute(PrintStyledContent(": ".bold().red()))?;
        }
        stdout()
            .execute(PrintStyledContent(y.reset()))?
            .execute(MoveToNextLine(1))?;
    }
    let (_x, y) = position().unwrap();
    stdout().execute(MoveTo(0, max(y, pos.1) + 1))?;
    Ok(())
}
