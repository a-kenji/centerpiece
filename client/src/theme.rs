use anyhow::Context;

/// The desktop's preferred color scheme, as reported by the XDG settings portal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

/// Concrete colors and egui theme for the active [`ColorScheme`].
pub struct ResolvedColors {
    pub text: egui::Color32,
    pub background: egui::Color32,
    pub egui_theme: egui::Theme,
}

impl ColorScheme {
    /// Reads the desktop color-scheme preference from the XDG settings portal.
    ///
    /// Falls back to [`ColorScheme::Dark`] on any failure (no portal running,
    /// dbus error, unexpected value type, "no preference", or "prefer dark").
    /// Only an explicit "prefer light" yields [`ColorScheme::Light`].
    pub fn detect() -> Self {
        detect_from_portal().unwrap_or(ColorScheme::Dark)
    }

    /// Resolves the configured colors into the palette for this scheme.
    ///
    /// The light scheme reuses the dark configuration with text and background
    /// swapped, so a custom dark palette stays readable when inverted and the
    /// default black-on-nothing becomes black-on-white.
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

/// Maps the portal's `color-scheme` value to a [`ColorScheme`].
///
/// Per the freedesktop appearance spec: `0` = no preference, `1` = prefer dark,
/// `2` = prefer light. Anything but an explicit light preference stays dark.
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

/// The portal's `Read` wraps the value in a variant nested inside a variant, so
/// descend recursively until the integer is found.
fn refarg_to_u32(arg: &dyn dbus::arg::RefArg) -> Option<u32> {
    if let Some(value) = arg.as_u64() {
        return Some(value as u32);
    }
    arg.as_iter()?.next().and_then(refarg_to_u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_portal_values_to_schemes() {
        assert_eq!(color_scheme_from_u32(0), ColorScheme::Dark);
        assert_eq!(color_scheme_from_u32(1), ColorScheme::Dark);
        assert_eq!(color_scheme_from_u32(2), ColorScheme::Light);
        assert_eq!(color_scheme_from_u32(99), ColorScheme::Dark);
    }

    #[test]
    fn light_swaps_text_and_background() {
        let color = settings::ColorSettings::default();
        let dark = ColorScheme::Dark.resolve(&color);
        let light = ColorScheme::Light.resolve(&color);

        assert_eq!(dark.egui_theme, egui::Theme::Dark);
        assert_eq!(light.egui_theme, egui::Theme::Light);
        assert_eq!(light.text, dark.background);
        assert_eq!(light.background, dark.text);
    }
}
