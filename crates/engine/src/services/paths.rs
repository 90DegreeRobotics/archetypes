use std::path::PathBuf;

pub fn app_data_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("data")
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

    #[test]
    fn config_root_is_a_sibling_of_data_not_a_child() {
        let config = config_root();
        let data = app_data_root();
        assert_eq!(config.file_name().unwrap(), "config");
        assert_eq!(data.file_name().unwrap(), "data");
        assert_eq!(config.parent(), data.parent());
    }
}
