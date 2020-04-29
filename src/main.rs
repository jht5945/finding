extern crate argparse;
extern crate term;
extern crate term_size;
extern crate rust_util;

mod opt;
mod local_util;

use std::{ cell::Cell, path::Path, time::SystemTime, };
use opt::*;
use rust_util::{
    iff,
    XResult,
    new_box_error,
    util_file::*,
    util_size::*,
    util_msg::*,
};
use local_util::read_file_content;

const EMPTY: &str = "";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");

#[derive(Debug)]
struct MatchLine {
    line_number: usize,
    line_string: String,
}

impl MatchLine {
    fn new(line_number: usize, line_string: String) -> MatchLine {
        MatchLine { line_number, line_string, }
    }
}

fn print_version() {
    println!(r#"finding {} - {}
Copyright (C) 2019-2020 Hatter Jiang.
License MIT <https://opensource.org/licenses/MIT>

Written by Hatter Jiang"#, VERSION, &GIT_HASH[0..7]);
}

fn clear_n_print_message(mt: MessageType, message: &str) {
    print_lastline(EMPTY);
    print_message(mt, message);
}

fn find_huge_files(options: &Options, dir_path: &Path) {
    let total_file_count_cell = Cell::new(0_u64);
    let huge_file_count_cell = Cell::new(0_u64);
    let huge_file_size_cell = Cell::new(0_u64);
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| { // process file
        total_file_count_cell.replace(total_file_count_cell.get() + 1);
        let p_str = match p.to_str() {
            Some(s) => s, None => return,
        };
        match p.metadata() {
            Err(err) => if options.verbose {
                clear_n_print_message(MessageType::WARN, &format!("Read file {} meta failed: {}", p_str, err));
            },
            Ok(metadata) => {
                let len = metadata.len();
                if len >= options.parsed_huge_file_size {
                    huge_file_count_cell.replace(huge_file_count_cell.get() + 1);
                    huge_file_size_cell.replace(huge_file_size_cell.get() + 1);
                    clear_n_print_message(MessageType::OK, &format!("{} [{}]", p_str, get_display_size(len as i64)));
                }
            },
        }
    }, &|p| { // process path
        let p_str = match p.to_str() {
            Some(s) => s, None => return false,
        };
        if options.skip_link_dir && is_symlink(p) {
            if options.verbose {
                clear_n_print_message(MessageType::INFO, &format!("Skip link dir: {}", p_str));
            }
            return false;
        }
        print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10));
        true
    }).ok();
    clear_n_print_message(MessageType::OK, &format!("Total file count: {}, huge file count: {}, total huge file size: {}",
                                            total_file_count_cell.into_inner(),
                                            huge_file_count_cell.into_inner(),
                                            get_display_size(huge_file_size_cell.into_inner() as i64)));
}

fn match_lines(tag: &str, content: &str, options: &Options) -> bool {
    let search_text = &options.search_text;
    let lines = content.lines();
    let mut match_lines_vec = vec![];
    let mut line_no = 0usize;
    let the_search_text = &iff!(options.ignore_case, search_text.to_lowercase(), search_text.to_string());
    for ln in lines {
        if options.filter_large_line && ln.len() as u64 >= options.parsed_large_line_size {
            if options.verbose {
                clear_n_print_message(MessageType::INFO, &format!("Skip large line: {} bytes", ln.len()));
            }
            continue;
        }
        let matches = iff!(options.ignore_case, ln.to_lowercase().contains(the_search_text), ln.contains(the_search_text));
        let matches_line_content = match &options.filter_line_content {
            c if c.is_empty() => true,
            c => ln.contains(c),
        };
        if matches && matches_line_content {
            match_lines_vec.push(MatchLine::new(line_no, ln.to_string()));
        }
        line_no += 1;
    }

    if match_lines_vec.is_empty() {
        false
    } else {
        clear_n_print_message(MessageType::OK, &format!("Find in {}:", tag));
        for match_line in &match_lines_vec {
            print!("{}: ", match_line.line_number + 1);
            if options.ignore_case {
                println!("{}", match_line.line_string);
            } else {
                let ss: Vec<&str> = match_line.line_string.split(search_text).collect();
                for j in 0..ss.len() {
                    print!("{}", ss[j]);
                    if j < ss.len() - 1 {
                        print_color(Some(term::color::RED), true, search_text);
                    }
                }
                println!();
            }
        }
        true
    }
}

fn find_text_files(options: &Options, dir_path: &Path) {
    if options.search_text.is_empty() {
        print_message(MessageType::ERROR, "Param search_text cannot be empty.");
        return;
    }
    if options.ignore_case {
        print_message(MessageType::WARN, "Using ignore case mode, highlight print is disabled.");
    }
    let file_exts = match &options.file_ext {
        ext if ext.is_empty() => vec![],
        ext => ext.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| ".".to_owned() + s).collect(),
    };
    let total_file_count_cell   = Cell::new(0_u64);
    let scaned_file_count_cell  = Cell::new(0_u64);
    let matched_file_count_cell = Cell::new(0_u64);
    let total_dir_count_cell    = Cell::new(0_u64);
    let scaned_dir_count_cell   = Cell::new(0_u64);
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| { // process file
        total_file_count_cell.replace(total_file_count_cell.get() + 1);
        let p_str = match p.to_str() {
            Some(s) => s, None => return,
        };
        if !file_exts.is_empty() && !file_exts.iter().any(|file_ext| p_str.to_lowercase().ends_with(file_ext)) {
            return;
        }
        if !options.filter_file_name.is_empty() && !p_str.contains(options.filter_file_name.as_str()) {
            return;
        }
        let file_content = match read_file_content(p, options.parsed_large_text_file_size) {
            Ok(c) => c, Err(err) => {
                if options.verbose { clear_n_print_message(MessageType::WARN, &format!("Read file {} failed: {}", p_str, err)); }
                return;
            },
        };
        scaned_file_count_cell.replace(scaned_file_count_cell.get() + 1);
        if match_lines(p_str, &file_content, &options) {
            matched_file_count_cell.replace(matched_file_count_cell.get() + 1);
        }
    }, &|p| { // process path
        total_dir_count_cell.replace(total_dir_count_cell.get() + 1);
        let p_str = match p.to_str() {
            Some(s) => s, None => return false,
        };
        if (!options.scan_dot_git_dir) && p_str.ends_with("/.git") {
            if options.verbose { clear_n_print_message(MessageType::INFO, &format!("Skip .git dir: {}", p_str)); }
            return false;
        }
        if options.skip_target_dir && p_str.ends_with("/target") {
            if options.verbose { clear_n_print_message(MessageType::INFO, &format!("Skip target dir: {}", p_str)); }
            return false;
        }
        if options.skip_dot_dir && p_str.contains("/.") {
            if options.verbose { clear_n_print_message(MessageType::INFO, &format!("Skip dot(.) dir: {}", p_str)); }
            return false;
        }
        if options.skip_link_dir && is_symlink(p) {
            if options.verbose { clear_n_print_message(MessageType::INFO, &format!("Skip link dir: {}", p_str)); }
            return false;
        }
        scaned_dir_count_cell.replace(scaned_dir_count_cell.get() + 1);
        print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10));
        true
    }).ok();
    print_lastline(EMPTY);
    print_message(MessageType::OK, &format!("Total dir count: {}, scaned dir count: {}",
                                    total_dir_count_cell.into_inner(),
                                    scaned_dir_count_cell.into_inner()));
    print_message(MessageType::OK, &format!("Total file count: {}, scaned file count: {}, matched file count: {}",
                                    total_file_count_cell.into_inner(),
                                    scaned_file_count_cell.into_inner(),
                                    matched_file_count_cell.into_inner()));
}

fn main() -> XResult<()> {
    let options = Options::new_and_parse_args()?;
    
    if options.version {
        print_version();
        return Ok(());
    }

    let dir_path = match get_absolute_path(&options.dir) {
        Some(path) => path,
        None => return Err(new_box_error(&format!("Cannot find dir: {}", options.dir))),
    };
    let start = SystemTime::now();
    match options.target.as_str() {
        "huge" | "hugefile" => find_huge_files(&options, &dir_path),
        "text" => find_text_files(&options, &dir_path),
        others => return Err(new_box_error(&format!("Unknown command: {}", others))),
    }
    let cost_millis = SystemTime::now().duration_since(start.clone()).unwrap().as_millis();
    print_message(MessageType::OK, &format!("Finding finished, cost {} ms", cost_millis));
    Ok(())
}
