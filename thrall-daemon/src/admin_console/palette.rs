//! The accent palette and the background modes.
//!
//! The palette is the eight-bit colour set the portal has always shipped. The
//! footer picker and the theme payload both read it from here, so the colour a
//! visitor picks and the colour the administrator defaults to can never drift
//! apart.

/// How a surface chooses its background image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BgMode {
    /// No background image at all.
    Off,
    /// One of the shipped images, chosen per visit.
    Random,
    /// One fixed image, chosen by the administrator.
    Pinned,
}

impl BgMode {
    /// Parse a stored or submitted mode. Anything unknown falls back to random.
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "off" => BgMode::Off,
            "pin" => BgMode::Pinned,
            _ => BgMode::Random,
        }
    }

    /// The wire form of this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            BgMode::Off => "off",
            BgMode::Random => "random",
            BgMode::Pinned => "pin",
        }
    }
}

/// One entry of the eight-bit accent palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accent {
    pub name: &'static str,
    pub hex: &'static str,
    pub rgb: [u8; 3],
    pub hot: &'static str,
    pub hot_rgb: [u8; 3],
}

/// The shipped palette.
pub const ACCENTS: [Accent; 8] = [
    Accent {
        name: "Coral",
        hex: "#ff5555",
        rgb: [255, 85, 85],
        hot: "#ff9999",
        hot_rgb: [255, 153, 153],
    },
    Accent {
        name: "Orange",
        hex: "#ff7700",
        rgb: [255, 119, 0],
        hot: "#ffa040",
        hot_rgb: [255, 160, 64],
    },
    Accent {
        name: "Amber",
        hex: "#ffb000",
        rgb: [255, 176, 0],
        hot: "#ffd447",
        hot_rgb: [255, 212, 71],
    },
    Accent {
        name: "Yellow",
        hex: "#f1fa8c",
        rgb: [241, 250, 140],
        hot: "#ffffb0",
        hot_rgb: [255, 255, 176],
    },
    Accent {
        name: "Emerald",
        hex: "#50fa7b",
        rgb: [80, 250, 123],
        hot: "#a2ffbc",
        hot_rgb: [162, 255, 188],
    },
    Accent {
        name: "Cyan",
        hex: "#00e5ff",
        rgb: [0, 229, 255],
        hot: "#80f4ff",
        hot_rgb: [128, 244, 255],
    },
    Accent {
        name: "Blue",
        hex: "#3b82f6",
        rgb: [59, 130, 246],
        hot: "#7cb3ff",
        hot_rgb: [124, 179, 255],
    },
    Accent {
        name: "Violet",
        hex: "#bd93f9",
        rgb: [189, 147, 249],
        hot: "#d9b8ff",
        hot_rgb: [217, 184, 255],
    },
];

/// The accent used when nothing else is configured.
pub const DEFAULT_ACCENT: &str = "Emerald";

/// Look up a palette entry by name, ignoring case.
pub fn accent_by_name(name: &str) -> Option<&'static Accent> {
    ACCENTS
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name.trim()))
}
