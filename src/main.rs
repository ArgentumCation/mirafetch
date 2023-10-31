#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{anyhow, Result};
use arcstr::ArcStr;
use crossterm::{
    cursor::{position, MoveTo, MoveToColumn, MoveToNextLine},
    style::{Color, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType::All},
    ExecutableCommand,
};
use directories::ProjectDirs;
use mirafetch::{
    colorizer::{Colorizer, Default, Flag},
    config::Config,
    info::Info,
    util::{get_colorscheme, get_icon, AsciiArt},
};
use std::{cmp::max, fs, io::stdout, process::ExitCode, sync::Arc};
mod util;

fn main() -> anyhow::Result<std::process::ExitCode> {
    // Load Settings

    // load from TOML
    let proj_dir = ProjectDirs::from("", "", "Mirafetch");
    let settings = proj_dir
        .ok_or_else(|| {
            anyhow!(
                "Could not find a project directory for Mirafetch. Please report this as a bug.",
            )
        })
        .and_then(|dir| {
            let config_path = dir.config_dir().join("config.toml");
            if !config_path.exists() {
                return anyhow::Ok(Config::default());
            };
            let config_file = fs::read_to_string(config_path)?;
            toml::from_str::<Config>(&config_file).map_err(|err| {
                eprintln!("Invalid config: {err}");
                anyhow!(exitcode::CONFIG)
            })
        })?;

    // Get Color Scheme from archive
    let scheme: Option<Arc<[Color]>> = settings
        .scheme_name
        .as_ref()
        .map_or_else(|| None, |name| Some(get_colorscheme(name).unwrap()));
    let info = Info::default();
    let id: Box<str> = Box::from(info.id.as_ref());
    let info_vec = info.as_vec();

    let logo: AsciiArt = get_icon(settings.icon_name.as_ref().get_or_insert(&id))?;
    let colored_logo = colorize_logo(&settings, &scheme, &logo)?;

    // Show system info

    display(colored_logo, info_vec, &logo)?;
    Ok(ExitCode::from(exitcode::OK as u8))
}

fn colorize_logo(
    settings: &Config,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt,
) -> Result<Vec<crossterm::style::StyledContent<String>>, anyhow::Error> {
    let colorizer = scheme.as_ref().map_or_else(
        || Ok(Box::new(Default {}) as Box<dyn Colorizer>),
        |scheme| {
            settings.orientation.map_or_else(
                || Err(anyhow!("Missing Orientation")),
                |orientation| {
                    Ok(Box::new(Flag {
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
    info: Vec<(ArcStr, ArcStr)>,
    logo: &AsciiArt,
) -> Result<(), anyhow::Error> {
    stdout().execute(Clear(All))?.execute(MoveTo(0, 0))?;
    for line in icon {
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
