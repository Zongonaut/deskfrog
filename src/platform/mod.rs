#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{cursor_pos, emoji_font_path, enumerate_monitors, FrogWindow};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::{cursor_pos, emoji_font_path, enumerate_monitors, FrogWindow};
