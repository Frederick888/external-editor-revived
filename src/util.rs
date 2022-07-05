use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

use crate::model::thunderbird::Tab;

pub fn guess_self_path(arg0: &str) -> Result<PathBuf> {
    let path = PathBuf::from(arg0);
    if path.is_absolute() && path.exists() {
        return Ok(path);
    }

    let mut pwd = env::current_dir()?;
    pwd.push(&path);
    let absolute_path = pwd.canonicalize()?;
    if absolute_path.exists() {
        Ok(absolute_path)
    } else {
        Err(anyhow!(
            "Failed to determine program path: got {} but it doesn't exist",
            absolute_path.to_string_lossy()
        ))
    }
}

pub fn get_temp_filename(tab: &Tab) -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(format!("external_editor_revived_{}.eml", tab.id));
    temp_dir
}
