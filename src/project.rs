use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::io::Write;
use std::os::unix;
use std::path::PathBuf;
use std::time;

pub const CDTEST_ROOT_VAR: &str = "/var/tmp/cdtest";
pub const CDTEST_ROOT_TMP: &str = "/tmp/cdtest";
pub const DEFAULT_GC_DURATION: u64 = 604800 * 2; // 2 weeks
pub const CDTEST_TOML: &str = ".cdtest.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    /// Name of project directory
    pub name: String,
    /// Last cdtest project access time
    #[serde(with = "humantime_serde")]
    pub timestamp: time::SystemTime,
    /// Override already-set configuration
    #[serde(skip)]
    pub force_override: bool,
    /// Duration
    #[serde(with = "humantime_serde")]
    pub garbage_collection: time::Duration,
    /// Whether or directory is temp-only
    #[serde(skip)]
    pub tmp_only: bool,
    /// Whether or not this project already existed
    #[serde(skip)]
    pub existing: bool,
}

impl Context {
    pub fn new(project: &str) -> Self {
        // Try to initialize from var
        let var_dir = PathBuf::from(CDTEST_ROOT_VAR).join(project);
        if let Ok(proj) = Self::from_project_dir(&var_dir) {
            return proj;
        }
        // Try to initialize from tmp
        let tmp_dir = PathBuf::from(CDTEST_ROOT_TMP).join(project);
        if let Ok(mut proj) = Self::from_project_dir(&tmp_dir) {
            proj.tmp_only = true;
            return proj;
        }
        // Create new
        Self {
            name: project.to_string(),
            existing: false,
            ..Default::default()
        }
    }

    pub fn from_project_dir(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.is_dir() {
            return Err("Project is not directory!".into());
        }
        let proj_conf = path.join(CDTEST_TOML);
        Ok(toml::from_str(fs::read_to_string(proj_conf)?.as_str())?)
    }

    /// Home directory of project
    pub fn home(&self) -> PathBuf {
        match self.tmp_only {
            false => self.var_home(),
            true => self.tmp_home(),
        }
    }

    /// Var home of project
    pub fn var_home(&self) -> PathBuf {
        let mut var_home = PathBuf::from(CDTEST_ROOT_VAR);
        var_home = var_home.join(self.name.as_str());
        var_home
    }

    /// Tmp home of project
    pub fn tmp_home(&self) -> PathBuf {
        let mut tmp_home = PathBuf::from(CDTEST_ROOT_TMP);
        tmp_home = tmp_home.join(self.name.as_str());
        tmp_home
    }

    /// Initialize project
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.timestamp = time::SystemTime::now();
        let home = self.home();
        if !home.is_dir() {
            fs::create_dir(&home)?;
        }
        let tmp_home = self.tmp_home();
        if !self.tmp_only && !tmp_home.is_dir() {
            unix::fs::symlink(&home, &tmp_home)?;
        }
        self.write_out()?;
        Ok(())
    }

    /// Write project context out
    pub fn write_out(&self) -> Result<(), Box<dyn std::error::Error>> {
        let as_toml = toml::to_string(&self)?;
        let proj_file = self.home().join(CDTEST_TOML);
        let mut pf_io = fs::File::create(proj_file)?;
        pf_io.write_all(as_toml.as_bytes())?;
        Ok(())
    }

    /// Remove project...
    pub fn garbage_collect(&self) -> () {
        if let Ok(time_since) = self.timestamp.elapsed() {
            if time_since > self.garbage_collection {
                fs::remove_dir_all(self.home()).ok();
                fs::remove_dir_all(self.tmp_home()).ok();
            }
        }
    }
}

impl Default for Context {
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
