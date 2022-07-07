use std::env;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::model::thunderbird::Tab;

pub fn get_temp_filename(tab: &Tab) -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(format!("external_editor_revived_{}.eml", tab.id));
    temp_dir
}

#[inline]
pub fn error_message_with_path<T>(e: T, path: &Path) -> String
where
    T: Display,
{
    format!(
        "{}.\nYou can try recovering data from {}",
        e,
        path.to_string_lossy()
    )
}
