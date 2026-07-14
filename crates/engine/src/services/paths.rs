use std::path::PathBuf;

pub fn app_data_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("data")
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
}
