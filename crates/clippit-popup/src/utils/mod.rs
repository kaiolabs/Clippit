pub mod thumbnail;
pub mod focus;
pub mod theme;

pub use thumbnail::create_thumbnail;
pub use focus::{get_focused_window_id, simulate_paste_to_window};
pub use theme::{apply_theme, load_custom_css};
