use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local};
use humansize::{format_size, BINARY};
use users::get_user_by_uid;

pub struct FileInfo {
    pub permissions: String,
    pub size: String,
    pub owner: String,
    pub modified: String,
    pub name: String,
    pub is_dir: bool,
}

/// Gets detailed information about a file or directory entry
///
/// # Arguments
///
/// * `entry` - A reference to a directory entry to get information about
///
/// # Returns
///
/// Some(FileInfo) containing the file's metadata if successful, None if there was an error
pub fn get_file_info(entry: &fs::DirEntry) -> Option<FileInfo> {
    let metadata = entry.metadata().ok()?;
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();

    // Get permissions
    let mode = metadata.permissions().mode();
    let permissions = format!(
        "{}{}",
        if metadata.is_dir() { "d" } else { "." },
        format_mode(mode)
    );

    // Get file size
    let size = if metadata.is_dir() {
        "-".to_string()
    } else {
        format_size(metadata.len(), BINARY)
    };

    let owner = {
        let uid = metadata.uid();
        get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string())
    };

    // Get modification time
    let modified: DateTime<Local> = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| DateTime::from(SystemTime::now() - d))
        .unwrap_or_else(|| Local::now());

    let modified_str = modified.format("%b %e %H:%M").to_string();

    Some(FileInfo {
        permissions,
        size,
        owner,
        modified: modified_str,
        name: file_name.to_string(),
        is_dir: metadata.is_dir(),
    })
}

/// Formats Unix file permissions mode into rwx string representation
///
/// # Arguments
///
/// * `mode` - The Unix permissions mode as a u32 bitmask
///
/// # Returns
///
/// A string containing the rwx permissions for user, group and other (e.g. "rwxr-xr--")
fn format_mode(mode: u32) -> String {
    let user = (mode >> 6) & 0o7;
    let group = (mode >> 3) & 0o7;
    let other = mode & 0o7;

    format!(
        "{}{}{}",
        format_rwx(user),
        format_rwx(group),
        format_rwx(other)
    )
}

/// Formats a 3-bit Unix permission set into rwx string notation
///
/// # Arguments
///
/// * `bits` - 3 bits representing read, write, execute permissions
///
/// # Returns
///
/// A 3-character string containing 'r', 'w', 'x' for set bits or '-' for unset bits
fn format_rwx(bits: u32) -> String {
    let r = if bits & 0b100 != 0 { 'r' } else { '-' };
    let w = if bits & 0b010 != 0 { 'w' } else { '-' };
    let x = if bits & 0b001 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}

pub fn list_files_detailed(path: &str, show_hidden: bool) -> Vec<FileInfo> {
    let entries = fs::read_dir(path).expect("Unable to read directory");
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if show_hidden || !file_name.starts_with('.') {
                get_file_info(&entry)
            } else {
                None
            }
        })
        .collect()
}

/// Lists files in the specified directory
///
/// # Arguments
///
/// * `path` - Path to the directory to list files from
/// * `show_hidden` - Whether to include hidden files (those starting with .) in the listing
///
/// # Returns
///
/// A vector of filenames as strings
pub fn list_files(path: &str, show_hidden: bool) -> Vec<String> {
    let entries = fs::read_dir(path).expect("Unable to read directory");
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if show_hidden || !file_name.starts_with('.') {
                Some(file_name.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_list_files() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Unable to create temporary directory");
        let dir_path = temp_dir.path();

        // Create some files in the temporary directory
        File::create(dir_path.join("file1.txt")).expect("Unable to create file1.txt");
        File::create(dir_path.join("file2.txt")).expect("Unable to create file2.txt");
        File::create(dir_path.join(".hidden_file")).expect("Unable to create .hidden_file");
        fs::create_dir(dir_path.join("subdir")).expect("Unable to create subdir");

        // Test 1: show_hidden = false
        let files = list_files(dir_path.to_str().unwrap(), false);
        let mut expected_files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "subdir".to_string(),
        ];
        expected_files.sort();
        let mut files_sorted = files.clone();
        files_sorted.sort();
        assert_eq!(files_sorted, expected_files);

        // Test 2: show_hidden = true
        let files = list_files(dir_path.to_str().unwrap(), true);
        let mut expected_files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            ".hidden_file".to_string(),
            "subdir".to_string(),
        ];
        expected_files.sort();
        let mut files_sorted = files.clone();
        files_sorted.sort();
        assert_eq!(files_sorted, expected_files);
    }
}
