#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{anyhow, Ok, Result};
use arcstr::ArcStr;
use clap::Parser;
use crossterm::{
    cursor::{position, MoveTo, MoveToColumn, MoveToNextLine},
    style::{Color, PrintStyledContent, Stylize},
    ExecutableCommand,
};
use directories::ProjectDirs;
use mirafetch::{
    colorizer::{Colorizer, DefaultColors, FlagColors},
    config::{Config, Orientation},
    util::{get_colorscheme, get_icon, AsciiArt},
};
use std::{fmt::Display, fs, io::stdout, process::ExitCode, sync::Arc};
use std::{
    sync::mpsc,
    thread::{self},
};
mod info;
mod util;

fn main() -> anyhow::Result<std::process::ExitCode> {
    let settings = load_settings_file()?.with_config(Config::parse());
    let scheme = get_colorscheme_from_settings(&settings);

    let (tx, rx) = mpsc::channel();
    let id = info::get_id();
    let logo: AsciiArt = get_icon(get_os_id(&settings, &id))?;
    let colored_logo = colorize_logo(settings.orientation, &scheme, &logo)?;
    thread::spawn(move || {
        info::get_async(tx);
    });

    // Show system info
    display(colored_logo, rx, &logo).ok();

    Ok(ExitCode::SUCCESS)
}

fn get_os_id(settings: &Config, default: &impl ToString) -> impl ToString {
    settings
        .icon_name
        .as_ref()
        .unwrap_or(&default.to_string())
        .to_owned()
}

fn get_colorscheme_from_settings(settings: &Config) -> Option<Arc<[Color]>> {
    let scheme: Option<Arc<[Color]>> = settings
        .scheme_name
        .as_ref()
        .map_or_else(|| None, |name| Some(get_colorscheme(name)));
    scheme
}

fn load_settings_file() -> Result<Config, anyhow::Error> {
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
    Ok(settings)
}

fn colorize_logo(
    orientation: Option<Orientation>,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt,
) -> Result<impl IntoIterator<Item = crossterm::style::StyledContent<impl Display>>, anyhow::Error>
{
    let colorizer = match scheme {
        None => Box::new(DefaultColors {}) as Box<dyn Colorizer>,
        Some(colors) => Box::new(FlagColors {
            color_scheme: colors.clone(),
            orientation: orientation.ok_or_else(|| anyhow!("Missing Orientation"))?,
        }) as Box<dyn Colorizer>,
    };

    Ok(colorizer.colorize(logo))
}

/// Display the formatted logo and system information
///
/// # Errors
///
/// This function will return an error if the terminal settings (eg color, cursor position) cannot be modified
fn display(
    icon: impl IntoIterator<Item = crossterm::style::StyledContent<impl Display>>,
    info: impl IntoIterator<Item = (ArcStr, ArcStr)>,
    logo: &AsciiArt,
) -> Result<(), anyhow::Error> {
    let mut out = stdout();
    println!("");
    for line in icon {
        out /* .execute(ResetColor)?*/
            .execute(PrintStyledContent(line))?;
    }
    let pos = position()?;
    out.execute(MoveTo(0, pos.1 - logo.height))?;
    for (property, value) in info {
        out.execute(MoveToColumn(logo.width + 3))?
            .execute(PrintStyledContent(property.clone().bold().red()))?;
        if !property.is_empty() && !value.is_empty() {
            out.execute(PrintStyledContent(": ".bold().red()))?;
        }
        out.execute(PrintStyledContent(value.reset()))?
            .execute(MoveToNextLine(1))?;
        // sleep(Duration::from_secs(1));
    }
    //let (_x, y) = position()?;
    //out.execute(MoveTo(0, pos.1 + 1))?;
    println!("");
    Ok(())
}
