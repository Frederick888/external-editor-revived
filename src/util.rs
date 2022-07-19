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

pub fn is_extension_compatible(host_version: &str, extension_version: &str) -> bool {
    let host_version: Vec<&str> = host_version.split('.').collect();
    let extension_version: Vec<&str> = extension_version.split('.').collect();

    host_version.len() == 3
        && extension_version.len() == 3
        && host_version[0] == extension_version[0]
        && host_version[1] == extension_version[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_patch_version_diff_test() {
        let host_version = "1.0.0";
        let extension_version = "1.0.1-beta";
        let compatible = is_extension_compatible(host_version, extension_version);
        assert!(compatible);
    }

    #[test]
    fn extension_minor_version_diff_test() {
        let host_version = "1.0.0";
        let extension_version = "1.1.0";
        let compatible = is_extension_compatible(host_version, extension_version);
        assert!(!compatible);
    }

    #[test]
    fn malformed_extension_version_test() {
        let host_version = "1.0.0";
        let extension_version = "1.0.0.0";
        let compatible = is_extension_compatible(host_version, extension_version);
        assert!(!compatible);
    }
}
