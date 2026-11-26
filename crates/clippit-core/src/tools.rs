use std::path::PathBuf;

/// Path to run one of Clippit's companion programs (`clippit-correct`, ...).
///
/// `scripts/install.sh` puts them in `~/.local/bin`, which the GNOME session
/// does not always have in its PATH, so that copy wins. Other installs (the
/// Nix package, for one) only have them in the PATH: the bare name is
/// returned and `Command` looks it up there.
pub fn tool(name: &str) -> PathBuf {
    let local = dirs::home_dir()
        .unwrap_or_default()
        .join(".local/bin")
        .join(name);
    if local.exists() {
        local
    } else {
        PathBuf::from(name)
    }
}
