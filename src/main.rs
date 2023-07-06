/// # cdtest
/// Traverse and manage semi-temporary test directories
/// ## Dev
///
use clap::{arg, Command};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process;

const CDTEST_ROOT_VAR: &str = "/var/tmp/cdtest";
const CDTEST_ROOT_TMP: &str = "/tmp/cdtest";

fn initialize_cdtest() -> Result<(), Box<dyn std::error::Error>> {
    let cdtest_root_var = Path::new(CDTEST_ROOT_VAR);
    if !cdtest_root_var.is_dir() {
        fs::create_dir(cdtest_root_var)?;
    }
    let cdtest_root_tmp = Path::new(CDTEST_ROOT_TMP);
    if !cdtest_root_tmp.is_dir() {
        fs::create_dir(cdtest_root_tmp)?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectContext {
    /// Name of project directory
    name: String,
    /// Whether or directory is temp-only
    tmp_only: bool,
    /// TODO garbage collection!
    garbage_collection: bool,
}

impl ProjectContext {
    fn new(project: &str) -> Self {
        Self {
            name: project.to_string(),
            tmp_only: false,
            garbage_collection: false,
        }
    }

    /// Home directory of project
    fn home(&self) -> PathBuf {
        match self.tmp_only {
            false => self.var_home(),
            true => self.tmp_home(),
        }
    }

    /// Var home of project
    fn var_home(&self) -> PathBuf {
        let mut var_home = PathBuf::from(CDTEST_ROOT_VAR);
        var_home = var_home.join(self.name.as_str());
        var_home
    }

    /// Tmp home of project
    fn tmp_home(&self) -> PathBuf {
        let mut tmp_home = PathBuf::from(CDTEST_ROOT_TMP);
        tmp_home = tmp_home.join(self.name.as_str());
        tmp_home
    }

    /// Initialize project
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let home = self.home();
        if !home.is_dir() {
            fs::create_dir(&home)?;
        }
        let tmp_home = self.tmp_home();
        if !self.tmp_only && !tmp_home.is_dir() {
            unix::fs::symlink(&home, &tmp_home)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let matches = Command::new("cdtest")
        .version("0.1")
        .author("Harrison Hall")
        .about("Traverse and manage semi-temporary test directories")
        // .arg(arg!(--test <VALUE>).required(false))
        .arg(arg!(--tmp).required(false))
        .arg(arg!(<PROJECT>).required(false))
        .get_matches();

    initialize_cdtest()?;

    // Create project context
    let default_project_name = "test".to_string();
    let project_name = matches
        .get_one::<String>("PROJECT")
        .unwrap_or(&default_project_name);
    let mut project = ProjectContext::new(project_name);
    project.tmp_only = matches.get_flag("tmp");
    project.initialize()?;
    env::set_current_dir(project.home())?;

    // Create subshell in project directory
    let current_shell = env::var("SHELL").expect("$SHELL is not set");
    let mut subshell = process::Command::new(current_shell).spawn()?;
    subshell.wait()?;

    Ok(())
}
