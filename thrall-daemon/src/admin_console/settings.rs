//! The portal settings model: appearance controls and operational switches.
//!
//! Every knob an administrator can turn lives here. The model is persisted in
//! the roster as `@ui` records, so it travels with the state the daemon owns
//! end to end, and it is rendered into both the sign-in page and the dashboard.

use super::palette::{accent_by_name, BgMode, DEFAULT_ACCENT};

/// The number of background images the portal ships.
pub const BG_COUNT: u8 = 5;

/// Bounds on the reconciliation interval, in seconds.
pub const MIN_INTERVAL: u32 = 15;
pub const MAX_INTERVAL: u32 = 3600;
pub const DEFAULT_INTERVAL: u32 = 45;

/// Every administrator-controlled knob of the portal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalSettings {
    pub footer_upload: bool,
    pub footer_hubs: bool,
    pub footer_mesh: bool,
    pub footer_vault: bool,
    pub footer_clock: bool,
    pub palette: bool,
    pub accent: String,
    pub monochrome: bool,
    pub bg_login: BgMode,
    pub bg_login_index: u8,
    pub bg_dashboard: BgMode,
    pub bg_dashboard_index: u8,
    pub spotlight: bool,
    pub reduce_motion: bool,
    pub maintenance: bool,
    pub interval: u32,
}

impl Default for PortalSettings {
    fn default() -> Self {
        Self {
            footer_upload: true,
            footer_hubs: true,
            footer_mesh: true,
            footer_vault: true,
            footer_clock: true,
            palette: true,
            accent: DEFAULT_ACCENT.to_string(),
            monochrome: false,
            bg_login: BgMode::Random,
            bg_login_index: 1,
            bg_dashboard: BgMode::Random,
            bg_dashboard_index: 1,
            spotlight: true,
            reduce_motion: false,
            maintenance: false,
            interval: DEFAULT_INTERVAL,
        }
    }
}

impl PortalSettings {
    /// Resolve the configured accent, falling back to the default palette entry.
    pub fn accent_def(&self) -> &'static super::palette::Accent {
        accent_by_name(&self.accent)
            .unwrap_or(accent_by_name(DEFAULT_ACCENT).unwrap_or(&super::palette::ACCENTS[4]))
    }

    /// True when every footer data box is enabled.
    pub fn all_footer_on(&self) -> bool {
        self.footer_upload
            && self.footer_hubs
            && self.footer_mesh
            && self.footer_vault
            && self.footer_clock
    }

    /// Turn every footer data box on or off at once.
    pub fn set_footer_all(&mut self, on: bool) {
        self.footer_upload = on;
        self.footer_hubs = on;
        self.footer_mesh = on;
        self.footer_vault = on;
        self.footer_clock = on;
    }

    /// The background mode and image index for a surface.
    pub fn background(&self, sign_in: bool) -> (BgMode, u8) {
        if sign_in {
            (self.bg_login, self.bg_login_index)
        } else {
            (self.bg_dashboard, self.bg_dashboard_index)
        }
    }

    /// Clamp a background image index into the shipped range.
    pub fn clamp_index(index: u8) -> u8 {
        if index == 0 {
            1
        } else {
            index.min(BG_COUNT)
        }
    }

    /// Clamp a reconciliation interval into the supported range.
    pub fn clamp_interval(seconds: u32) -> u32 {
        seconds.clamp(MIN_INTERVAL, MAX_INTERVAL)
    }

    /// Apply one stored `@ui` record. Unknown keys are ignored.
    pub fn apply_record(&mut self, key: &str, value: &str) {
        let flag = || value.trim() == "1" || value.trim().eq_ignore_ascii_case("true");
        match key.trim() {
            "footer_upload" => self.footer_upload = flag(),
            "footer_hubs" => self.footer_hubs = flag(),
            "footer_mesh" => self.footer_mesh = flag(),
            "footer_vault" => self.footer_vault = flag(),
            "footer_clock" => self.footer_clock = flag(),
            "palette" => self.palette = flag(),
            "monochrome" => self.monochrome = flag(),
            "spotlight" => self.spotlight = flag(),
            "reduce_motion" => self.reduce_motion = flag(),
            "maintenance" => self.maintenance = flag(),
            "accent" => {
                if accent_by_name(value).is_some() {
                    self.accent = value.trim().to_string();
                }
            }
            "bg_login" => self.bg_login = BgMode::parse(value),
            "bg_dashboard" => self.bg_dashboard = BgMode::parse(value),
            "bg_login_index" => {
                self.bg_login_index = Self::clamp_index(value.trim().parse().unwrap_or(1));
            }
            "bg_dashboard_index" => {
                self.bg_dashboard_index = Self::clamp_index(value.trim().parse().unwrap_or(1));
            }
            "interval" => {
                self.interval =
                    Self::clamp_interval(value.trim().parse().unwrap_or(DEFAULT_INTERVAL));
            }
            _ => {}
        }
    }

    /// Render the model as the `@ui` records that persist it.
    pub fn records(&self) -> Vec<(String, String)> {
        let flag = |b: bool| if b { "1".to_string() } else { "0".to_string() };
        vec![
            ("footer_upload".into(), flag(self.footer_upload)),
            ("footer_hubs".into(), flag(self.footer_hubs)),
            ("footer_mesh".into(), flag(self.footer_mesh)),
            ("footer_vault".into(), flag(self.footer_vault)),
            ("footer_clock".into(), flag(self.footer_clock)),
            ("palette".into(), flag(self.palette)),
            ("accent".into(), self.accent.clone()),
            ("monochrome".into(), flag(self.monochrome)),
            ("bg_login".into(), self.bg_login.as_str().to_string()),
            ("bg_login_index".into(), self.bg_login_index.to_string()),
            (
                "bg_dashboard".into(),
                self.bg_dashboard.as_str().to_string(),
            ),
            (
                "bg_dashboard_index".into(),
                self.bg_dashboard_index.to_string(),
            ),
            ("spotlight".into(), flag(self.spotlight)),
            ("reduce_motion".into(), flag(self.reduce_motion)),
            ("maintenance".into(), flag(self.maintenance)),
            ("interval".into(), self.interval.to_string()),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_records_round_trip_through_apply() {
        let s = PortalSettings {
            footer_mesh: false,
            accent: "Coral".to_string(),
            bg_dashboard: BgMode::Pinned,
            bg_dashboard_index: 4,
            interval: 120,
            ..Default::default()
        };
        let mut back = PortalSettings::default();
        for (k, v) in s.records() {
            back.apply_record(&k, &v);
        }
        assert_eq!(s, back);
    }

    #[test]
    fn test_records_are_clamped_and_validated() {
        let mut s = PortalSettings::default();
        s.apply_record("interval", "99999");
        assert_eq!(s.interval, MAX_INTERVAL);
        s.apply_record("bg_login_index", "99");
        assert_eq!(s.bg_login_index, BG_COUNT);
        s.apply_record("accent", "Not-A-Colour");
        assert_eq!(s.accent, DEFAULT_ACCENT);
        s.apply_record("unknown_key", "1");
    }
}
