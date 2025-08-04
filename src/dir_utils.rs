use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local};
use humansize::{format_size, BINARY};
use tabled::Tabled;
use users::{get_user_by_uid, get_group_by_gid};

#[derive(Tabled)]
pub struct FileInfo {
    pub permissions: String,
    pub links: String,
    pub owner: String,
    pub group: String,
    pub size: String,
    pub modified: String,
    pub name: String,
    #[tabled(skip)]
    pub is_dir: bool,
    #[tabled(skip)]
    pub file_size: u64,
    #[tabled(skip)]
    pub modified_time: SystemTime,
}

/// Gets detailed information about a file or directory entry
///
/// # Arguments
///
/// * `entry` - A reference to a directory entry to get information about
/// * `human_readable` - Whether to format file sizes in human-readable format
///
/// # Returns
///
/// Some(FileInfo) containing the file's metadata if successful, None if there was an error
pub fn get_file_info(entry: &fs::DirEntry, human_readable: bool) -> Option<FileInfo> {
    let metadata = entry.metadata().ok()?;
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();

    // Get permissions
    let mode = metadata.permissions().mode();
    let permissions = format!(
        "{}{}",
        if metadata.is_dir() { "d" } else { "-" },
        format_mode(mode)
    );

    // Get number of hard links
    let links = metadata.nlink().to_string();

    // Get file size
    let file_size = metadata.len();
    let size = if metadata.is_dir() {
        "-".to_string()
    } else if human_readable {
        format_size(file_size, BINARY)
    } else {
        file_size.to_string()
    };

    let owner = {
        let uid = metadata.uid();
        get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string())
    };

    let group = {
        let gid = metadata.gid();
        get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string())
    };

    // Get modification time
    let modified_time = metadata.modified().unwrap_or(SystemTime::now());
    let modified: DateTime<Local> = modified_time
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| DateTime::from(UNIX_EPOCH + d))
        .unwrap_or_else(|| Local::now());

    let modified_str = modified.format("%b %e %H:%M").to_string();

    Some(FileInfo {
        permissions,
        links,
        owner,
        group,
        size,
        modified: modified_str,
        name: file_name.to_string(),
        is_dir: metadata.is_dir(),
        file_size,
        modified_time,
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

/// Adds file type indicator to filename based on file type
///
/// # Arguments
///
/// * `name` - The filename
/// * `metadata` - File metadata to determine type
///
/// # Returns
///
/// Filename with appropriate indicator appended
fn add_file_type_indicator(name: &str, metadata: &fs::Metadata) -> String {
    let indicator = if metadata.is_dir() {
        "/"
    } else if metadata.permissions().mode() & 0o111 != 0 {
        "*" // executable
    } else {
        ""
    };
    format!("{}{}", name, indicator)
}

pub fn list_files_detailed(path: &str, show_hidden: bool, almost_all: bool, human_readable: bool, sort_time: bool, sort_size: bool, reverse: bool, unsorted: bool) -> Vec<FileInfo> {
    let entries = fs::read_dir(path).expect("Unable to read directory");
    let mut files: Vec<FileInfo> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            // Handle hidden files and . .. filtering
            if !show_hidden && file_name.starts_with('.') {
                return None;
            }
            if almost_all && (file_name == "." || file_name == "..") {
                return None;
            }

            get_file_info(&entry, human_readable)
        })
        .collect();

    // Apply sorting unless unsorted is specified
    if !unsorted {
        if sort_time {
            files.sort_by(|a, b| {
                if reverse {
                    a.modified_time.cmp(&b.modified_time)
                } else {
                    b.modified_time.cmp(&a.modified_time)
                }
            });
        } else if sort_size {
            files.sort_by(|a, b| {
                if reverse {
                    a.file_size.cmp(&b.file_size)
                } else {
                    b.file_size.cmp(&a.file_size)
                }
            });
        } else {
            // Default alphabetical sort
            files.sort_by(|a, b| {
                if reverse {
                    b.name.cmp(&a.name)
                } else {
                    a.name.cmp(&b.name)
                }
            });
        }
    }

    files
}

/// Lists files in the specified directory
///
/// # Arguments
///
/// * `path` - Path to the directory to list files from
/// * `show_hidden` - Whether to include hidden files (those starting with .) in the listing
/// * `almost_all` - Whether to exclude . and .. from listing
/// * `classify` - Whether to add file type indicators
/// * `sort_time` - Whether to sort by modification time
/// * `sort_size` - Whether to sort by file size
/// * `reverse` - Whether to reverse the sort order
/// * `unsorted` - Whether to skip sorting entirely
///
/// # Returns
///
/// A vector of filenames as strings
pub fn list_files(path: &str, show_hidden: bool, almost_all: bool, classify: bool, sort_time: bool, sort_size: bool, reverse: bool, unsorted: bool) -> Vec<String> {
    let entries = fs::read_dir(path).expect("Unable to read directory");
    let mut files: Vec<(String, fs::Metadata, SystemTime)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            // Handle hidden files and . .. filtering
            if !show_hidden && file_name.starts_with('.') {
                return None;
            }
            if almost_all && (file_name == "." || file_name == "..") {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let modified_time = metadata.modified().unwrap_or(SystemTime::now());
            
            let display_name = if classify {
                add_file_type_indicator(&file_name, &metadata)
            } else {
                file_name.to_string()
            };

            Some((display_name, metadata, modified_time))
        })
        .collect();

    // Apply sorting unless unsorted is specified
    if !unsorted {
        if sort_time {
            files.sort_by(|a, b| {
                if reverse {
                    a.2.cmp(&b.2)
                } else {
                    b.2.cmp(&a.2)
                }
            });
        } else if sort_size {
            files.sort_by(|a, b| {
                if reverse {
                    a.1.len().cmp(&b.1.len())
                } else {
                    b.1.len().cmp(&a.1.len())
                }
            });
        } else {
            // Default alphabetical sort
            files.sort_by(|a, b| {
                if reverse {
                    b.0.cmp(&a.0)
                } else {
                    a.0.cmp(&b.0)
                }
            });
        }
    }

    files.into_iter().map(|(name, _, _)| name).collect()
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

        // Test 1: show_hidden = false, classify = false
        let files = list_files(dir_path.to_str().unwrap(), false, false, false, false, false, false, false);
        let mut expected_files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "subdir".to_string(),
        ];
        expected_files.sort();
        let mut files_sorted = files.clone();
        files_sorted.sort();
        assert_eq!(files_sorted, expected_files);

        // Test 2: show_hidden = true, classify = false
        let files = list_files(dir_path.to_str().unwrap(), true, false, false, false, false, false, false);
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

        // Test 3: classify = true (should add / to directories)
        let files = list_files(dir_path.to_str().unwrap(), false, false, true, false, false, false, false);
        let mut expected_files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "subdir/".to_string(),
        ];
        expected_files.sort();
        let mut files_sorted = files.clone();
        files_sorted.sort();
        assert_eq!(files_sorted, expected_files);
    }

    #[test]
    fn test_file_info_permissions() {
        // Test the permission format fix (should use '-' not '.')
        let temp_dir = tempdir().expect("Unable to create temporary directory");
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path).expect("Unable to create test file");
        
        let entry = fs::read_dir(temp_dir.path())
            .expect("Unable to read directory")
            .find(|e| e.as_ref().unwrap().file_name() == "test_file.txt")
            .expect("File not found")
            .expect("Unable to get directory entry");
            
        let file_info = get_file_info(&entry, false).expect("Unable to get file info");
        assert!(file_info.permissions.starts_with('-'), "Regular file should start with '-' not '.'");
    }

    #[test]
    fn test_sorting() {
        let temp_dir = tempdir().expect("Unable to create temporary directory");
        let dir_path = temp_dir.path();

        // Create files with different names
        File::create(dir_path.join("c.txt")).expect("Unable to create c.txt");
        File::create(dir_path.join("a.txt")).expect("Unable to create a.txt");
        File::create(dir_path.join("b.txt")).expect("Unable to create b.txt");

        // Test default alphabetical sorting
        let files = list_files(dir_path.to_str().unwrap(), false, false, false, false, false, false, false);
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);

        // Test reverse sorting
        let files = list_files(dir_path.to_str().unwrap(), false, false, false, false, false, true, false);
        assert_eq!(files, vec!["c.txt", "b.txt", "a.txt"]);

        // Test unsorted (should maintain original order from filesystem)
        let files = list_files(dir_path.to_str().unwrap(), false, false, false, false, false, false, true);
        // Just ensure we get all files (order might vary)
        let mut sorted_files = files.clone();
        sorted_files.sort();
        assert_eq!(sorted_files, vec!["a.txt", "b.txt", "c.txt"]);
    }
}
