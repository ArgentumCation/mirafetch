#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Config {
    pub scheme_name: Option<Box<str>>,
    pub orientation: Option<Orientation>,
    pub icon_name: Option<Box<str>>,
}

impl Config {
    /// Builder method to add an icon to config
    #[must_use]
    pub fn with_icon(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(Into::<String>::into(icon_name).into_boxed_str());
        self
    }
    /// Create new struct containing user settings
    #[must_use]
    pub fn new(
        scheme_name: Option<impl ToString>,
        orientation: Option<Orientation>,
        icon_name: Option<impl ToString>,
    ) -> Self {
        Self {
            scheme_name: scheme_name.map(|x| x.to_string().into_boxed_str()),
            orientation,
            icon_name: icon_name.map(|x| x.to_string().into_boxed_str()),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
