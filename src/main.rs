#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{anyhow, Result};
use arcstr::ArcStr;
use clap::Parser;
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
use std::{cmp::max, fmt::Display, fs, io::stdout, process::ExitCode, sync::Arc};
mod util;

fn main() -> anyhow::Result<std::process::ExitCode> {
    let settings = Config::parse();
    let settings = load_settings_file(settings)?;
    let scheme = get_colorscheme_from_settings(&settings);

    let info = Info::default();
    let id = info.id.clone();
    let info_vec = info.as_vec();
    let logo: AsciiArt = get_icon(get_os_id(&settings, id.as_str()))?;
    let colored_logo = colorize_logo(&settings, &scheme, &logo)?;

    // Show system info
    display(colored_logo, info_vec, &logo)?;

    Ok(ExitCode::SUCCESS)
}

fn get_os_id<'a>(settings: &'a Config, default: impl Into<&'a str>) -> impl Into<&str> {
    settings
        .icon_name
        .as_ref()
        .map_or_else(|| default.into(), |name| name.as_ref())
}

fn get_colorscheme_from_settings(settings: &Config) -> Option<Arc<[Color]>> {
    let scheme: Option<Arc<[Color]>> = settings
        .scheme_name
        .as_ref()
        .map_or_else(|| None, |name| Some(get_colorscheme(name.as_ref())));
    scheme
}

fn load_settings_file(cmd_args: Config) -> Result<Config, anyhow::Error> {
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
    // Merge config file options with arguments
    Ok(Config {
        scheme_name: cmd_args.scheme_name.or(settings.scheme_name),
        orientation: cmd_args.orientation.or(settings.orientation),
        icon_name: cmd_args.icon_name.or(settings.icon_name),
    })
}

fn colorize_logo(
    settings: &Config,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt,
) -> Result<impl IntoIterator<Item = crossterm::style::StyledContent<impl Display>>, anyhow::Error>
{
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
    icon: impl IntoIterator<Item = crossterm::style::StyledContent<impl Display>>,
    info: impl IntoIterator<Item = (ArcStr, ArcStr)>,
    logo: &AsciiArt,
) -> Result<(), anyhow::Error> {
    stdout().execute(Clear(All))?.execute(MoveTo(0, 0))?;
    // panic!();
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
