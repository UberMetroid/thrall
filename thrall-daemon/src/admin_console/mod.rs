//! The admin console: portal appearance settings and operational controls.
//!
//! `settings` holds the model every knob lives in, `theme` turns that model
//! into the payload the browser receives, `appearance` and `operations` render
//! the two new sections of the Admin tab, and `actions` handles the forms
//! that change them.
//!
//! The model travels with the rest of the daemon state, so a portal restored
//! from its roster comes back with the same appearance and the same
//! operational posture it was left in.

pub mod actions;
pub mod appearance;
pub mod operations;
pub mod palette;
pub mod settings;
pub mod theme;

#[cfg(test)]
mod console_tests;

pub use palette::{Accent, BgMode, ACCENTS};
pub use settings::PortalSettings;
pub use theme::{body_attrs, theme_css};
