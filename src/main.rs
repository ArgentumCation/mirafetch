#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![allow(unused_imports)]
#![warn(clippy::style)]

use anyhow::{ Result, anyhow };
use crossterm::{
    cursor::{ MoveTo, MoveToColumn, MoveToNextLine },
    style::{ PrintStyledContent, Stylize, Color },
    terminal::{ Clear, ClearType::All },
    ExecutableCommand,
};
use mirafetch::{
    util::{ AsciiArt, get_icon, get_colorscheme },
    GayColorizer,
    Config,
    Info,
    DefaultColorizer,
    Colorizer,
};

// mod linuxinfo;

// use rkyv::{ validation::validators::check_archived_root, Deserialize, Infallible };

use std::{ io::stdout, sync::Arc };
mod util;

fn main() -> anyhow::Result<()> {
    // Load Settings
    // todo: load from TOML
    let settings = Config::new(
        None,
        None,
        None,
        Some("Windows".to_string()) //Todo: determine distro and change this to None
    );

    // Get Color Scheme from archive

    let scheme: Option<Arc<[Color]>> = match settings.scheme_name {
        Some(ref x) => get_colorscheme(&x).ok(),
        None => None,
    };
    let logo: AsciiArt = get_icon(&settings.icon_name).unwrap();
    let colored_logo = colorize_logo(&settings, &scheme, &logo)?;

    // Get system info
    let info = Info::new().as_vec();

    display(colored_logo, info, logo)?;
    Ok(())
}

fn colorize_logo(
    settings: &Config,
    scheme: &Option<Arc<[Color]>>,
    logo: &AsciiArt
) -> Result<Vec<crossterm::style::StyledContent<String>>, anyhow::Error> {
    let colorizer = if settings.gay {
        if let Some(ref scheme) = *scheme {
            if let Some(orientation) = settings.orientation {
                Ok(
                    Box::new(GayColorizer {
                        color_scheme: scheme.clone(),
                        orientation,
                    }) as Box<dyn Colorizer>
                )
            } else {
                Err(anyhow!("Missing Orientation"))
            }
        } else {
            Err(anyhow!("Missing Scheme"))
        }
    } else {
        Ok(Box::new(DefaultColorizer {}) as Box<dyn Colorizer>)
    };
    Ok(colorizer?.colorize(logo))
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
