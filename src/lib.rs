//! Kit de interfaz egui de la suite Iurefficient (IureDesk, IureProject): tema y densidades,
//! iconos vectoriales, lista virtual densa, paleta de comandos y controles pequeños. Ver README.md.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod a11y;
pub mod fuzzy;
pub mod icons;
pub mod lang;
pub mod list;
pub mod palette;
pub mod system;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use icons::Icon;
pub use lang::{Lang, pick};
pub use theme::ThemeMode;
pub use tokens::{Density, Palette};
pub use widgets::avatar::avatar;
pub use widgets::badge::{BadgeKind, badge, count_badge};
pub use widgets::banner::{Banner, BannerResponse, Severity, Toasts};
pub use widgets::empty_state::EmptyState;
pub use widgets::keycap::keycap;
pub use widgets::nav_rail::{NavItem, nav_rail, sidebar_item};
pub use widgets::search_field::SearchField;
pub use widgets::segmented::segmented;
pub use widgets::split::{self, PaneKind};
pub use widgets::{primary_button, section_label};
