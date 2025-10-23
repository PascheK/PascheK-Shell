//! Visual components used by the TUI screens.
//!
//! Each component exposes a `render` function and small helpers to mutate their
//! local state. Components are intentionally dumb; all navigation and business
//! logic lives in the parent `tui` module and `state` module.
pub mod status;
pub mod terminal;
pub mod logs;
pub mod home;
pub mod explorer;
pub mod editor;