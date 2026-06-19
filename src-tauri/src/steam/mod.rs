//! Steam integration: locating the install, reading accounts, and (later) switching.

pub mod accounts;
pub mod avatar;
pub mod registry;
pub mod switch;
pub mod vdf;

pub use accounts::{list_accounts, Account};
