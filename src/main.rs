use std::path::Path;

use args::Args;
use clap::Parser;

pub mod args;
pub mod dir_utils;

/// Lists files and directories with formatting based on command line arguments
///
/// # Arguments
///
/// * `path` - Path to list contents from
/// * `args` - Command line arguments controlling listing format options
fn list_directory(path: &str, args: &Args) {
    if args.long {
        // Long format listing
        let files = dir_utils::list_files_detailed(path, args.all);
        for file in files {
            println!(
                "{} {} {} {} {}",
                file.permissions, file.owner, file.size, file.modified, file.name
            )
        }
    } else if args.recursive {
        // Recursive listing
        list_recursive(path, args.all);
    } else {
        // Short listing
        let files = dir_utils::list_files(path, args.all);
        for file in files {
            println!("{}  ", file);
        }
        println!();
    }
}

/// Recursively lists files and directories starting from the given path
///
/// # Arguments
///
/// * `path` - Path to start listing from
/// * `show_hidden` - Whether to show hidden files (starting with .)
fn list_recursive(path: &str, show_hidden: bool) {
    println!("\n{}:", path);
    let files = dir_utils::list_files(path, show_hidden);
    for file in &files {
        println!("{}  ", file);
    }
    println!();

    // Recursively list subdirectories
    for file in files {
        let full_path = Path::new(path).join(&file);
        if full_path.is_dir() {
            list_recursive(full_path.to_str().unwrap(), show_hidden);
        }
    }
}

fn main() {
    let args = Args::parse();
    list_directory(&args.path, &args);
}
