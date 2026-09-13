use std::path::PathBuf;

pub fn app_data_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("data")
}

/// Where Bevy's asset server reads from. **The single source of truth for that decision.**
///
/// A debug build reads the repository's `assets/` directly, so source edits appear without a copy
/// step. A release build reads `assets` **beside the executable**.
///
/// That second half is the part worth being exact about, because it is not what it looks like.
/// `AssetPlugin.file_path` was set to the relative string `"assets"`, and Bevy resolves a
/// relative asset path against the **executable's directory, not the working directory**. The
/// two are the same for an installed build and different for a release binary run from the
/// repository -- which is how the capture harness runs it, and the engine log says so plainly:
/// `Path not found: <repo>/target/release/assets/manifested/seeded_probe.glb`.
///
/// So this returns an absolute path in both profiles and `main.rs` hands it straight to Bevy.
/// A relative path here means two components can each be "right" about a different directory,
/// which is exactly how `resolve_asset` came to report a file as present that the asset server
/// could not open.
pub fn asset_root() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("assets")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("assets")))
            .unwrap_or_else(|| PathBuf::from("assets"))
    }
}

/// The Windows Metabolism contract's config tree, sibling to `data`. Settings live here
/// rather than under `app_data_root()` so a future data wipe does not also discard the
/// player's control/audio preferences.
pub fn config_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_data_root_returns_expected_path() {
        let path = app_data_root();
        assert!(path.to_string_lossy().contains("NeuroCognica"));
        assert!(path.to_string_lossy().contains("Archetypes"));
    }

    /// Bevy resolves a relative `AssetPlugin.file_path` against the executable's directory.
    /// Returning a relative path here lets the asset server and the ledger's resolver each be
    /// "right" about a different directory, which is how a manifested object came to be reported
    /// present while the asset server could not open it.
    #[test]
    fn the_asset_root_is_absolute_so_two_components_cannot_disagree_about_it() {
        let root = asset_root();
        assert!(root.is_absolute(), "{} is relative", root.display());
        assert_eq!(root.file_name().unwrap(), "assets");
    }

    #[test]
    fn config_root_is_a_sibling_of_data_not_a_child() {
        let config = config_root();
        let data = app_data_root();
        assert_eq!(config.file_name().unwrap(), "config");
        assert_eq!(data.file_name().unwrap(), "data");
        assert_eq!(config.parent(), data.parent());
    }
}
