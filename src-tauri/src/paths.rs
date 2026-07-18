//! Portable-aware locations owned by VoxWeave.

use std::path::{Path, PathBuf};

pub const PORTABLE_MARKER: &str = "voxweave.portable";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePaths {
    pub is_portable: bool,
    pub data_dir: PathBuf,
}

impl StoragePaths {
    pub fn from_exe_dir(exe_dir: &Path) -> Result<Self, String> {
        let is_portable = exe_dir.join(PORTABLE_MARKER).is_file();
        let data_dir = if is_portable {
            exe_dir.join("data")
        } else {
            dirs_next::config_dir()
                .ok_or_else(|| "Cannot determine user config directory".to_string())?
                .join("VoxWeave")
        };
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create VoxWeave data directory {}: {e}", data_dir.display()))?;
        Ok(Self { is_portable, data_dir })
    }

    pub fn current() -> Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| format!("Cannot resolve executable path: {e}"))?;
        let dir = exe.parent().ok_or_else(|| "Executable has no parent directory".to_string())?;
        Self::from_exe_dir(dir)
    }

    pub fn config_path(&self) -> PathBuf { self.data_dir.join("config.json") }
    pub fn models_dir(&self) -> PathBuf { self.data_dir.join("models") }
    pub fn logs_dir(&self) -> PathBuf { self.data_dir.join("logs") }

    /// Persist managed portable models relative to `data`; preserve installed absolute paths.
    pub fn persist_model_path(&self, path: &Path) -> Result<String, String> {
        if self.is_portable {
            let relative = path.strip_prefix(&self.data_dir)
                .map_err(|_| "Managed portable model path must remain under data/models".to_string())?;
            Ok(relative.to_string_lossy().replace('\\', "/"))
        } else {
            Ok(path.to_string_lossy().into_owned())
        }
    }

    pub fn resolve_model_path(&self, stored: &str) -> Result<PathBuf, String> {
        let path = Path::new(stored);
        if self.is_portable && !path.is_absolute() {
            let resolved = self.data_dir.join(path);
            if !resolved.starts_with(self.models_dir()) { return Err("Portable model path must stay under data/models".to_string()); }
            Ok(resolved)
        } else { Ok(path.to_path_buf()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_selects_portable_paths_and_relative_models() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join(PORTABLE_MARKER), "").unwrap();
        let paths = StoragePaths::from_exe_dir(temp.path()).unwrap();
        assert!(paths.is_portable);
        assert_eq!(paths.config_path(), temp.path().join("data/config.json"));
        assert_eq!(paths.logs_dir(), temp.path().join("data/logs"));
        let model = paths.models_dir().join("ggml-tiny.bin");
        assert_eq!(paths.persist_model_path(&model).unwrap(), "models/ggml-tiny.bin");
        assert_eq!(paths.resolve_model_path("models/ggml-tiny.bin").unwrap(), model);
    }
}
