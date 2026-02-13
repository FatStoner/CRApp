pub mod afterhour;
pub mod charactercard;
pub mod crave;
pub mod detection;
pub mod generic;
pub mod girlfriendgpt;
pub mod janitor;
pub mod spicychat;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-exports
pub use types::{ParsedCharacterData, ParsedLorebookData, ParsedLorebookEntry};
pub(crate) use utils::find_next_value_index;
pub use utils::{parse_clipboard, parse_crappbook_json};

// Wrapper for spicychat to keep public API if used elsewhere, or just re-export
pub use charactercard::{parse_png_card, parse_v2_card};
pub use spicychat::parse_spicychat_lorebook;
