/// # cdtest
/// Traverse and manage semi-temporary test directories
/// ## Dev
///
use clap::{arg, Command};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::vec::Vec;

use crate::project::Context;

mod project;

/// Initialize cdtest directories
fn initialize_cdtest() -> Result<(), Box<dyn std::error::Error>> {
    let cdtest_root_var = Path::new(project::CDTEST_ROOT_VAR);
    if !cdtest_root_var.is_dir() {
        fs::create_dir(cdtest_root_var)?;
    }
    let cdtest_root_tmp = Path::new(project::CDTEST_ROOT_TMP);
    if !cdtest_root_tmp.is_dir() {
        fs::create_dir(cdtest_root_tmp)?;
    }

    Ok(())
}

/// Parse all current project directories
fn parse_all_projects() -> Vec<project::Context> {
    let mut all_projects = Vec::<project::Context>::new();
    // Iterate var
    if let Ok(dir_iter) = fs::read_dir(project::CDTEST_ROOT_VAR) {
        for proj_path in dir_iter {
            if let Ok(proj_dir) = proj_path {
                let proj_dir = proj_dir.path();
                if !proj_dir.is_dir() {
                    continue;
                }
                if let Ok(proj_ctx) = Context::from_project_dir(&proj_dir) {
                    all_projects.push(proj_ctx);
                }
            }
        }
    }
    // Iterate tmp
    if let Ok(dir_iter) = fs::read_dir(project::CDTEST_ROOT_TMP) {
        for proj_path in dir_iter {
            if let Ok(proj_dir) = proj_path {
                let proj_dir = proj_dir.path();
                if proj_dir.is_symlink() {
                    continue;
                }
                if !proj_dir.is_dir() {
                    continue;
                }
                if let Ok(proj_ctx) = Context::from_project_dir(&proj_dir) {
                    all_projects.push(proj_ctx);
                }
            }
        }
    }

    all_projects
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let matches = Command::new("cdtest")
        .version("0.1")
        .author("Harrison Hall")
        .about("Traverse and manage semi-temporary test directories")
        .arg(arg!(--override "Override project settings (if existing)").required(false))
        .arg(arg!(--tmp "Exist only in memory").required(false))
        .arg(arg!(--gc <DURATION> "Set garbage collection duration").required(false))
        .arg(arg!(<PROJECT> "Project name").required(false))
        .get_matches();

    initialize_cdtest()?;

    // Create project context
    let default_project_name = "test".to_string();
    let project_name = matches
        .get_one::<String>("PROJECT")
        .unwrap_or(&default_project_name);
    let mut new_project = project::Context::new(project_name);
    new_project.force_override = *matches
        .get_one::<bool>("override")
        .unwrap_or(&new_project.force_override);
    if new_project.force_override || !new_project.existing {
        new_project.tmp_only = matches.get_flag("tmp");
        new_project.garbage_collection = match matches.get_one::<String>("gc") {
            Some(human_gc) => human_gc.parse::<humantime::Duration>()?.into(),
            None => new_project.garbage_collection,
        };
    }
    new_project.initialize()?;
    new_project.write_out()?;

    // Parse other projects
    let all_projects = parse_all_projects();
    for other_project in all_projects {
        other_project.garbage_collect();
    }

    // Create subshell in project directory
    env::set_current_dir(new_project.home())?;
    let current_shell = env::var("SHELL").expect("$SHELL is not set");
    let mut subshell = process::Command::new(current_shell).spawn()?;
    subshell.wait()?;

    Ok(())
}
