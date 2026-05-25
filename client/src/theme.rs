use anyhow::Context;

/// The desktop's preferred color scheme, as reported by the XDG settings portal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

/// Colors for the active [`ColorScheme`].
pub struct ResolvedColors {
    pub text: egui::Color32,
    pub background: egui::Color32,
    pub egui_theme: egui::Theme,
}

impl ColorScheme {
    /// Reads the desktop color-scheme preference from the XDG settings portal.
    pub fn detect() -> Self {
        detect_from_portal().unwrap_or(ColorScheme::Dark)
    }

    pub fn resolve(self, color: &settings::ColorSettings) -> ResolvedColors {
        let configured_text = settings::hexcolor(&color.text);
        let configured_background = settings::hexcolor(&color.background);
        match self {
            ColorScheme::Dark => ResolvedColors {
                text: configured_text,
                background: configured_background,
                egui_theme: egui::Theme::Dark,
            },
            ColorScheme::Light => ResolvedColors {
                text: configured_background,
                background: configured_text,
                egui_theme: egui::Theme::Light,
            },
        }
    }
}

fn color_scheme_from_u32(value: u32) -> ColorScheme {
    match value {
        2 => ColorScheme::Light,
        _ => ColorScheme::Dark,
    }
}

fn detect_from_portal() -> anyhow::Result<ColorScheme> {
    let connection = dbus::blocking::Connection::new_session()?;
    let proxy = connection.with_proxy(
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        std::time::Duration::from_millis(500),
    );
    let (value,): (dbus::arg::Variant<Box<dyn dbus::arg::RefArg>>,) = proxy.method_call(
        "org.freedesktop.portal.Settings",
        "Read",
        ("org.freedesktop.appearance", "color-scheme"),
    )?;
    let scheme = refarg_to_u32(&value.0).context("portal color-scheme value was not an integer")?;
    Ok(color_scheme_from_u32(scheme))
}

fn refarg_to_u32(arg: &dyn dbus::arg::RefArg) -> Option<u32> {
    if let Some(value) = arg.as_u64() {
        return Some(value as u32);
    }
    arg.as_iter()?.next().and_then(refarg_to_u32)
}
