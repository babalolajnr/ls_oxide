use std::path::Path;

use args::Args;
use clap::Parser;
use tabled::{settings::Style, Table};

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
        let show_hidden = args.all || args.almost_all;
        let files = dir_utils::list_files_detailed(
            path, 
            show_hidden, 
            args.almost_all, 
            args.human_readable, 
            args.sort_time, 
            args.sort_size, 
            args.reverse, 
            args.unsorted
        );
        let table = Table::new(files).with(Style::blank()).to_string();
        println!("{}", table)
    } else if args.recursive {
        // Recursive listing
        let show_hidden = args.all || args.almost_all;
        list_recursive(path, show_hidden, args.almost_all, args.classify, args.sort_time, args.sort_size, args.reverse, args.unsorted, args.one_per_line);
    } else {
        // Short listing
        let show_hidden = args.all || args.almost_all;
        let files = dir_utils::list_files(
            path, 
            show_hidden, 
            args.almost_all, 
            args.classify, 
            args.sort_time, 
            args.sort_size, 
            args.reverse, 
            args.unsorted
        );
        
        if args.one_per_line {
            for file in files {
                println!("{}", file);
            }
        } else {
            for file in files {
                print!("{}  ", file);
            }
            println!();
        }
    }
}

/// Recursively lists files and directories starting from the given path
///
/// # Arguments
///
/// * `path` - Path to start listing from
/// * `show_hidden` - Whether to show hidden files (starting with .)
/// * `almost_all` - Whether to exclude . and .. from listing
/// * `classify` - Whether to add file type indicators
/// * `sort_time` - Whether to sort by modification time
/// * `sort_size` - Whether to sort by file size
/// * `reverse` - Whether to reverse the sort order
/// * `unsorted` - Whether to skip sorting entirely
/// * `one_per_line` - Whether to list one file per line
fn list_recursive(path: &str, show_hidden: bool, almost_all: bool, classify: bool, sort_time: bool, sort_size: bool, reverse: bool, unsorted: bool, one_per_line: bool) {
    println!("\n{}:", path);
    let files = dir_utils::list_files(path, show_hidden, almost_all, classify, sort_time, sort_size, reverse, unsorted);
    
    if one_per_line {
        for file in &files {
            println!("{}", file);
        }
    } else {
        for file in &files {
            print!("{}  ", file);
        }
        println!();
    }

    // Recursively list subdirectories
    for file in files {
        // Remove file type indicator to get actual filename for path construction
        let clean_filename = if classify && (file.ends_with('/') || file.ends_with('*')) {
            &file[..file.len() - 1]
        } else {
            &file
        };
        
        let full_path = Path::new(path).join(clean_filename);
        if full_path.is_dir() {
            list_recursive(full_path.to_str().unwrap(), show_hidden, almost_all, classify, sort_time, sort_size, reverse, unsorted, one_per_line);
        }
    }
}

fn main() {
    let args = Args::parse();
    
    // If only one path and it's the default ".", list it without header
    if args.paths.len() == 1 && args.paths[0] == "." {
        list_directory(&args.paths[0], &args);
    } else {
        // Multiple paths, show headers for each
        for (i, path) in args.paths.iter().enumerate() {
            if i > 0 {
                println!(); // Add blank line between multiple path outputs
            }
            if args.paths.len() > 1 {
                println!("{}:", path);
            }
            list_directory(path, &args);
        }
    }
}
