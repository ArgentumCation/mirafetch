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
use directories::ProjectDirs;
use mirafetch::{
    colorizer::{Colorizer, DefaultColorizer, FlagColorizer},
    config::Config,
    info::Info,
    util::{get_colorscheme, get_icon, AsciiArt},
};

use std::{cmp::max, fs, io::stdout, sync::Arc};
mod util;

fn main() -> anyhow::Result<()> {
    // Load Settings
    let info = Info::default();
    let id: Box<str> = Box::from(info.id.as_ref());
    let info_vec = info.as_vec();
    // todo: load from TOML
    let settings: Config = ProjectDirs::from("", "", "Mirafetch")
        .and_then(
            // || Config::default().with_icon(id.as_ref()),
            |proj_dir| -> Option<Config> {
                let path = proj_dir.config_dir().join("config.toml");
                let conf_file = fs::read_to_string(path).ok()?;
                toml::from_str(&(conf_file))
                    .map_err(|err| println!("{err}"))
                    .ok()
            },
        )
        .or_else(|| Some(Config::default()))
        .unwrap();
    // Get Color Scheme from archive
    let scheme: Option<Arc<[Color]>> = settings.orientation.and(
        settings
            .scheme_name
            .as_ref()
            .and_then(|x| get_colorscheme(x).ok()),
    );
    let logo: AsciiArt = get_icon(settings.icon_name.as_ref().get_or_insert(&id))?;
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
    let colorizer = scheme.as_ref().map_or_else(
        || Ok(Box::new(DefaultColorizer {}) as Box<dyn Colorizer>),
        |scheme| {
            settings.orientation.map_or_else(
                || Err(anyhow!("Missing Orientation")),
                |orientation| {
                    Ok(Box::new(FlagColorizer {
                        color_scheme: scheme.clone(),
                        orientation,
                    }) as Box<dyn Colorizer>)
                },
            )
        },
    );

    Ok(colorizer?.colorize(logo))
}

/// Display the formatted logo and system information
///
/// # Errors
///
/// This function will return an error if the terminal settings (eg color, cursor position) cannot be modified
fn display(
    icon: Vec<crossterm::style::StyledContent<String>>,
    info: Vec<(String, String)>,
    logo: &AsciiArt,
) -> Result<(), anyhow::Error> {
    stdout().execute(Clear(All))?.execute(MoveTo(0, 0))?;
    for line in icon.into_iter().rev() {
        stdout() /* .execute(ResetColor)?*/
            .execute(PrintStyledContent(line))?;
    }
    let pos = position()?;
    stdout().execute(MoveTo(0, 0))?;
    for line in info {
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
    let (_x, y) = position()?;
    stdout().execute(MoveTo(0, max(y, pos.1) + 1))?;
    Ok(())
}
