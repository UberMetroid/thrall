//! Turn the settings model into the payload the browser receives.
//!
//! The portal's Content-Security-Policy forbids inline script, so the client
//! cannot be handed a configuration object. It is handed data attributes on
//! the body instead, and the colour variables arrive as an appended `:root`
//! block, which the browser resolves after the stylesheet's own defaults.

use super::palette::{Accent, BgMode};
use super::settings::PortalSettings;

/// Perceived brightness of a colour, used to build the monochrome variant.
fn luminance(rgb: [u8; 3]) -> u32 {
    (u32::from(rgb[0]) * 299 + u32::from(rgb[1]) * 587 + u32::from(rgb[2]) * 114) / 1000
}

/// The colour pair the theme drives: the base phosphor and its hot highlight.
fn theme_colors(a: &Accent, monochrome: bool) -> ([u8; 3], [u8; 3]) {
    if !monochrome {
        return (a.rgb, a.hot_rgb);
    }
    let g = luminance(a.rgb).clamp(40, 220) as u8;
    let hot = (u32::from(g) + 48).min(255) as u8;
    ([g, g, g], [hot, hot, hot])
}

/// The `:root` override that carries the administrator's accent colour.
fn accent_block(main: [u8; 3], hot: [u8; 3]) -> String {
    let (r, g, b) = (main[0], main[1], main[2]);
    let (hr, hg, hb) = (hot[0], hot[1], hot[2]);
    format!(
        ":root{{--phosphor:rgb({r},{g},{b});--accent:rgb({r},{g},{b});--accent-rgb:{r},{g},{b};\
         --phosphor-dim:rgba({r},{g},{b},0.12);--phosphor-glow:rgba({r},{g},{b},0.28);\
         --border-accent:rgba({r},{g},{b},0.38);--hot:rgb({hr},{hg},{hb});--hot-rgb:{hr},{hg},{hb};\
         --spotlight-border:rgba({r},{g},{b},0.52);--spotlight-glow:rgba({hr},{hg},{hb},0.92);\
         --spotlight-inner:rgba({r},{g},{b},0.045);}}"
    )
}

/// The inline CSS appended to the portal stylesheet.
pub fn theme_css(settings: &PortalSettings) -> String {
    let (main, hot) = theme_colors(settings.accent_def(), settings.monochrome);
    accent_block(main, hot)
}

/// The body attributes the client reads to honour the administrator's choices.
///
/// `sign_in` selects the surface: the sign-in page and the dashboard carry
/// their own background settings.
pub fn body_attrs(settings: &PortalSettings, sign_in: bool) -> String {
    let (mode, index) = settings.background(sign_in);
    let bg = match mode {
        BgMode::Off => "off",
        BgMode::Random => "random",
        BgMode::Pinned => "pin",
    };
    let accent = settings.accent_def().name.to_ascii_lowercase();
    format!(
        " data-accent-default=\"{accent}\" data-palette=\"{p}\" data-bg=\"{bg}\" \
         data-bg-index=\"{index}\" data-reduce-motion=\"{m}\"",
        p = u8::from(settings.palette),
        m = u8::from(settings.reduce_motion),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::admin_console::palette::BgMode;

    #[test]
    fn test_body_attrs_carry_the_surface_background() {
        let s = PortalSettings {
            bg_login: BgMode::Pinned,
            bg_login_index: 3,
            bg_dashboard: BgMode::Off,
            ..Default::default()
        };
        let login = body_attrs(&s, true);
        let dash = body_attrs(&s, false);
        assert!(login.contains("data-bg=\"pin\"") && login.contains("data-bg-index=\"3\""));
        assert!(dash.contains("data-bg=\"off\""));
        assert!(login.contains("data-accent-default=\"emerald\""));
    }

    #[test]
    fn test_monochrome_replaces_the_colour_with_a_grey() {
        let s = PortalSettings {
            monochrome: true,
            ..Default::default()
        };
        let css = theme_css(&s);
        assert!(css.contains("--accent:rgb("));
        assert!(!css.contains("#50fa7b"));
    }
}
