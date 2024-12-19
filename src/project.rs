//! Temporary project handling.

use super::*;

pub const CDTEST_ROOT_VAR: &str = "/var/tmp/cdtest";
pub const CDTEST_ROOT_TMP: &str = "/tmp/cdtest";
pub const DEFAULT_GC_DURATION: u64 = 604800 * 2; // 2 weeks  // TODO: once
pub const CDTEST_TOML: &str = ".cdtest.toml";

/// Temporary project configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// Name of temporary project. This is used to name the project directory.
    pub name: String,
    /// Last cdtest project access time.
    #[serde(with = "humantime_serde")]
    pub timestamp: time::SystemTime,
    /// Override already-set configuration.
    #[serde(skip)]
    pub force_override: bool,
    /// Duration until collected as garbage. This is reset on access from cdtest.
    #[serde(with = "humantime_serde")]
    pub garbage_collection: time::Duration,
    /// Whether project is temp-only.
    #[serde(skip)]
    pub tmp_only: bool,
    /// Whether or not this project already existed.
    #[serde(skip)]
    pub existing: bool,
}

impl Project {
    /// Construct new temporary project context.
    pub fn new(project_name: impl AsRef<str>) -> Self {
        let project_name = project_name.as_ref();
        // Try to initialize from var
        let var_dir = PathBuf::from(CDTEST_ROOT_VAR).join(project_name);
        if let Ok(proj) = Self::from_project_dir(&var_dir) {
            return proj;
        }
        // Try to initialize from tmp
        let tmp_dir = PathBuf::from(CDTEST_ROOT_TMP).join(project_name);
        if let Ok(mut proj) = Self::from_project_dir(&tmp_dir) {
            proj.tmp_only = true;
            return proj;
        }
        // Create new
        Self {
            name: project_name.to_string(),
            existing: false,
            ..Default::default()
        }
    }

    /// Read project context from the path itself.
    pub fn from_project_dir(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(Error::ProjectDirectoryInvalid);
        }
        let proj_conf = path.join(CDTEST_TOML);
        let file_as_str: String =
            std::fs::read_to_string(proj_conf.clone()).map_err(|_| Error::ParseFailedPath {
                from: proj_conf.to_str().unwrap_or("<invalid>").into(),
            })?;
        let project = toml::from_str(&file_as_str).map_err(|_| Error::ParseFailedToml {
            from: file_as_str.into(),
        })?;
        Ok(project)
    }

    /// The home directory of project.
    pub fn home(&self) -> PathBuf {
        match self.tmp_only {
            false => self.var_home(),
            true => self.tmp_home(),
        }
    }

    /// Var home of project.
    pub fn var_home(&self) -> PathBuf {
        let mut var_home = PathBuf::from(CDTEST_ROOT_VAR);
        var_home = var_home.join(self.name.as_str());
        var_home
    }

    /// Tmp home of project.
    pub fn tmp_home(&self) -> PathBuf {
        let mut tmp_home = PathBuf::from(CDTEST_ROOT_TMP);
        tmp_home = tmp_home.join(self.name.as_str());
        tmp_home
    }

    /// Initialize temporary project directory.
    pub fn initialize(&mut self) -> Result<(), Error> {
        self.timestamp = time::SystemTime::now();
        let home = self.home();
        if !home.is_dir() {
            std::fs::create_dir(&home).map_err(|_| Error::ProjectSetupFailed)?;
        }
        let tmp_home = self.tmp_home();
        if !self.tmp_only && !tmp_home.is_dir() {
            unix::fs::symlink(&home, &tmp_home).map_err(|_| Error::ProjectSetupFailed)?;
        }
        self.write_out()?;
        Ok(())
    }

    /// Write project context out to file.
    pub fn write_out(&self) -> Result<(), Error> {
        let as_toml = toml::to_string(&self).map_err(|_| Error::WriteOutFailed)?;
        let proj_file = self.home().join(CDTEST_TOML);
        let mut pf_io = std::fs::File::create(proj_file).map_err(|_| Error::WriteOutFailed)?;
        pf_io
            .write_all(as_toml.as_bytes())
            .map_err(|_| Error::WriteOutFailed)?;
        Ok(())
    }

    /// Collect garbage, removing project if necessary.
    pub fn garbage_collect(&self) -> () {
        if let Ok(time_since) = self.timestamp.elapsed() {
            if time_since > self.garbage_collection {
                std::fs::remove_dir_all(self.home()).ok();
                std::fs::remove_dir_all(self.tmp_home()).ok();
            }
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            timestamp: time::SystemTime::now(),
            force_override: false,
            tmp_only: false,
            existing: false,
            garbage_collection: time::Duration::from_secs(DEFAULT_GC_DURATION),
        }
    }
}
