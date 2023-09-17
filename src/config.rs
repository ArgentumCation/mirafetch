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
        // CR: Not sure if this would work but it might be better to specify the desired type
        // and then just call into instead of using the absolute whatever it's called
        // Also, why impl Into<String> instead of just impl ToString like below?
        self.icon_name = Some(Into::<String>::into(icon_name).into_boxed_str());
        self
    }
    /// Create new struct containing user settings
    // CR: Why have all the things be required in the new function instead of just 
    // deriving default and using with_attr() funcs for all of them? Also, why would you need 
    // start with one icon in new and then change it with with_icon?
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
