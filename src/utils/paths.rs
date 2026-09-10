use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;

const QUALIFIER: &str = "io.github";
const ORGANIZATION: &str = "Washu";
const APPLICATION: &str = "washu";

/// Filesystem locations owned by Washū, derived from the current user's XDG
/// configuration instead of hard-coded home-directory paths.
#[derive(Clone, Debug)]
pub struct AppPaths {
    data_dir: PathBuf,
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .context("discovering XDG application directories")?;

        Ok(Self {
            data_dir: dirs.data_local_dir().to_path_buf(),
            config_dir: dirs.config_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
        })
    }

    #[cfg(test)]
    pub fn for_testing(root: &Path) -> Self {
        Self {
            data_dir: root.join("data"),
            config_dir: root.join("config"),
            cache_dir: root.join("cache"),
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        for directory in [&self.data_dir, &self.config_dir, &self.cache_dir] {
            fs::create_dir_all(directory)
                .with_context(|| format!("creating {}", directory.display()))?;
        }
        Ok(())
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
    pub fn database_file(&self) -> PathBuf {
        self.data_dir.join("washu.db")
    }
    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }
}

#[cfg(test)]
mod tests {
    use super::AppPaths;
    use std::path::PathBuf;

    #[test]
    fn database_file_is_stored_below_data_directory() {
        let paths = AppPaths {
            data_dir: PathBuf::from("/tmp/washu-data"),
            config_dir: PathBuf::from("/tmp/washu-config"),
            cache_dir: PathBuf::from("/tmp/washu-cache"),
        };

        assert_eq!(
            paths.database_file(),
            PathBuf::from("/tmp/washu-data/washu.db")
        );
    }

    #[test]
    fn backups_are_stored_below_data_directory() {
        let paths = AppPaths {
            data_dir: PathBuf::from("/tmp/washu-data"),
            config_dir: PathBuf::from("/tmp/washu-config"),
            cache_dir: PathBuf::from("/tmp/washu-cache"),
        };

        assert_eq!(
            paths.backups_dir(),
            PathBuf::from("/tmp/washu-data/backups")
        );
    }
}
