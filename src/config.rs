use clap::{Parser, ValueEnum};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Parser, Eq, PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(short, long)]
    pub scheme_name: Option<String>,
    #[arg(value_enum, short, long)]
    pub orientation: Option<Orientation>,
    #[arg(short, long)]
    pub icon_name: Option<String>,
}

impl Config {
    /// Builder method to add an icon to config
    #[must_use]
    pub fn with_icon(self, icon_name: impl Into<String>) -> Self {
        Self {
            scheme_name: self.scheme_name,
            icon_name: Some(Into::<String>::into(icon_name)),
            orientation: self.orientation,
        }
    }
    #[must_use]
    pub fn with_scheme_name(self, scheme_name: impl Into<String>) -> Self {
        Self {
            scheme_name: Some(Into::<String>::into(scheme_name)),
            icon_name: self.icon_name,
            orientation: self.orientation,
        }
    }
    #[must_use]
    pub fn with_orientation(self, orientation: &Orientation) -> Self {
        Self {
            scheme_name: self.scheme_name,
            icon_name: self.icon_name,
            orientation: Some(*orientation),
        }
    }
    /// Create new struct containing user settings
    #[must_use]
    pub fn new(
        scheme_name: Option<impl ToString>,
        orientation: Option<Orientation>,
        icon_name: Option<impl ToString>,
    ) -> Self {
        Self {
            scheme_name: scheme_name.map(|x| x.to_string()),
            orientation,
            icon_name: icon_name.map(|x| x.to_string()),
        }
    }
    #[must_use]
    pub fn with_config(self, other: Self) -> Self {
        Self {
            scheme_name: other.scheme_name.or(self.scheme_name),
            orientation: other.orientation.or(self.orientation),
            icon_name: other.icon_name.or(self.icon_name),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone, ValueEnum, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
