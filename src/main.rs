#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{anyhow, Context, Ok, Result};
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
    config::{Config, Orientation},
    info::Info,
    util::{get_colorscheme, get_icon, AsciiArt},
};
use std::{cmp::max, fmt::Display, fs, io::stdout, process::ExitCode, sync::Arc};
mod util;

fn main() -> anyhow::Result<std::process::ExitCode> {
    let mut settings = load_settings()?;
    let scheme = get_colorscheme_from_settings(&settings);

    let info = Info::default();
    settings.icon_name = settings
        .icon_name
        .or(Some(info.id.clone().to_string().into_boxed_str()));
    let info_vec = info.as_vec();
    let logo: AsciiArt = get_icon(&settings.icon_name.expect("Missing icon name"))?;
    let colored_logo = colorize_logo(settings.orientation.as_ref(), &scheme, &logo)?;

    // Show system info
    display(colored_logo, info_vec, &logo)?;

    Ok(ExitCode::SUCCESS)
}

fn get_colorscheme_from_settings(settings: &Config) -> Option<Arc<[Color]>> {
    let scheme: Option<Arc<[Color]>> = settings
        .scheme_name
        .as_ref()
        .map_or_else(|| None, |name| Some(get_colorscheme(name.as_ref())));
    scheme
}

fn load_settings() -> anyhow::Result<Config> {
    let cli_args = Config::parse();
    let dir = ProjectDirs::from("", "", "Mirafetch")
        .expect("Could not find a project directory for Mirafetch. Please report this as a bug.");
    let config_path = dir.config_dir().join("config.toml");
    let mut settings = Config::default();
    if config_path.exists() {
        let config_file = fs::read_to_string(config_path)?;
        settings = toml::from_str::<Config>(&config_file)
            .map_err(|err| anyhow!(exitcode::CONFIG).context(err))?;
    }
    // Merge config file options with arguments
    settings.scheme_name = cli_args.scheme_name.or(settings.scheme_name);
    settings.orientation = cli_args.orientation.or(settings.orientation);
    settings.icon_name = cli_args.icon_name.or(settings.icon_name);
    Ok(settings)
}

fn colorize_logo(
    orientation: Option<&Orientation>,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt,
) -> Result<impl IntoIterator<Item = crossterm::style::StyledContent<impl Display>>, anyhow::Error>
{
    let colorizer = match scheme {
        None => Box::new(Default {}) as Box<dyn Colorizer>,
        Some(colors) => Box::new(Flag {
            color_scheme: colors.clone(),
            orientation: *orientation.ok_or(anyhow!("Missing Orientation"))?,
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
    out.execute(Clear(All))?.execute(MoveTo(0, 0))?;
    // panic!();
    for line in icon {
        out /* .execute(ResetColor)?*/
            .execute(PrintStyledContent(line))?;
    }
    let pos = position()?;
    out.execute(MoveTo(0, 0))?;
    for (property, value) in info {
        out.execute(MoveToColumn(logo.width + 3))?
            .execute(PrintStyledContent(property.clone().bold().red()))?;
        if !property.is_empty() && !value.is_empty() {
            out.execute(PrintStyledContent(": ".bold().red()))?;
        }
        out.execute(PrintStyledContent(value.reset()))?
            .execute(MoveToNextLine(1))?;
    }
    let (_x, y) = position()?;
    out.execute(MoveTo(0, max(y, pos.1) + 1))?;
    Ok(())
}
