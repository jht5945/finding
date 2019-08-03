extern crate argparse;
extern crate term;
extern crate term_size;
extern crate rust_util;

mod opt;

use std::{
    cell::RefCell,
    fs::File,
    path::Path,
    io::prelude::*,
    time::SystemTime,
};

use opt::*;
use rust_util::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");

fn print_version() {
    print!(r#"finding {} - {}
Copyright (C) 2019 Hatter Jiang.
License MIT <https://opensource.org/licenses/MIT>

Written by Hatter Jiang
"#, VERSION, &GIT_HASH[0..7]);
}

fn get_term_width() -> Option<usize> {
    match term_size::dimensions() {
        None => None,
        Some((w, _h)) => Some(w),
    }
}

// thanks https://blog.csdn.net/star_xiong/article/details/89401149
fn find_char_boundary(s: &str, index: usize) -> usize {
    if s.len() <= index {
        return index;
    }
    let mut new_index = index;
    while !s.is_char_boundary(new_index) {
        new_index += 1;
    }
    new_index
}

fn get_term_width_message(message: &str, left: usize) -> String {
    match get_term_width() {
        None => message.to_string(),
        Some(w) => {
            let len = message.len();
            if w > len {
               return message.to_string();
            }
            let mut s = String::new();
            s.push_str(&message[0..find_char_boundary(&message, w-10-5-left)]);
            s.push_str("[...]");
            s.push_str(&message[find_char_boundary(&message, len-10)..]);
            s
        },
    }
}

fn find_huge_files(options: &Options, dir_path: &Path) {
    let total_file_count_cell = RefCell::new(0u64);
    let huge_file_count_cell = RefCell::new(0u64);
    let huge_file_size_cell = RefCell::new(0u64);
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| {
        total_file_count_cell.replace_with(|&mut c| c + 1);
        match p.metadata() {
            Err(err) => {
                if options.verbose {
                    let p_str = p.to_str();
                    if p_str.is_some() {
                        print_lastline("");
                        print_message(MessageType::WARN, &format!("Read file {} meta failed: {}", p_str.unwrap(), err));
                    }
                }
                return;
            },
            Ok(metadata) => {
                let len = metadata.len();
                if len >= options.parsed_huge_file_size {
                    huge_file_count_cell.replace_with(|&mut c| c + 1);
                    huge_file_size_cell.replace_with(|&mut c| c + len);
                    match p.to_str() {
                        None => (),
                        Some(p_str) => {
                            print_lastline("");
                            print_message(MessageType::OK, &format!("{} [{}]", p_str, get_display_size(len as i64)));
                        },
                    }
                }
            },
        }
    }, &|p| {
        match p.to_str() {
            None => (),
            Some(p_str) => {
                if options.skip_link_dir && is_symlink(p) {
                    if options.verbose {
                        print_lastline("");
                        print_message(MessageType::INFO, &format!("Skip link dir: {}", p_str));
                    }
                    return false;
                }
                print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10))
            },
        }
        true
    }).unwrap_or(());
    print_lastline("");
    print_message(MessageType::OK, &format!("Total file count: {}, huge file count: {}, huge file size: {}",
                                    total_file_count_cell.into_inner(),
                                    huge_file_count_cell.into_inner(),
                                    huge_file_size_cell.into_inner()))
}

fn read_file_content(file: &Path, large_file_len: u64) -> XResult<String> {
    if ! file.exists() {
        return Err(new_box_error(&format!("File not exists: {:?}", file)));
    }
    if ! file.is_file() {
        return Err(new_box_error(&format!("File is not file: {:?}", file)));
    }
    let file_len = file.metadata()?.len();
    if file_len > large_file_len {
        return Err(new_box_error(&format!("File too large: {:?}, len: {}", file, file_len)));
    }
    let mut f = File::open(file)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    Ok(content)
}

#[derive(Debug)]
struct MatchLine {
    line_number: usize,
    line_string: String,
}

impl MatchLine {
    fn new(l_no: usize, l_str: String) -> MatchLine {
        MatchLine {
            line_number: l_no,
            line_string: l_str,
        }
    }
}

fn match_lines(tag: &str, content: &String, options: &Options) -> bool {
    let search_text = &options.search_text;
    let lines = content.lines();
    let mut match_lines_vec = vec![];
    let mut l_no = 0usize;
    let the_search_text = &match options.ignore_case {
        true => search_text.to_lowercase(),
        false => search_text.to_string(),
    };
    for ln in lines {
        if options.filter_large_line && ln.len() as u64 >= options.parsed_large_line_size {
            if options.verbose {
                print_lastline("");
                print_message(MessageType::INFO, &format!("Skip large line: {} bytes", ln.len()));
            }
            continue;
        }
        let matches = match options.ignore_case {
            true => ln.to_lowercase().contains(the_search_text),
            false => ln.contains(the_search_text),
        };
        if matches {
            match_lines_vec.push(MatchLine::new(l_no, ln.to_string()));
        }
        l_no += 1;
    }

    let mut matches_any = false;
    if match_lines_vec.len() > 0 {
        print_lastline("");
        print_message(MessageType::OK, &format!("Find in {}:", tag));
        for i in 0..match_lines_vec.len() {
            print!("{}: ", match_lines_vec[i].line_number + 1);
            match options.ignore_case {
                true => println!("{}", match_lines_vec[i].line_string),
                false => {
                    let ss: Vec<&str> = match_lines_vec[i].line_string.split(search_text).collect();
                    for j in 0..ss.len() {
                        print!("{}", ss[j]);
                        if j < ss.len() -1 {
                            print_color(Some(term::color::RED), true, search_text);
                        }
                    }
                    println!();
                },
            }
        }
        matches_any = true;
    }
    matches_any
}

fn find_text_files(options: &Options, dir_path: &Path) {
    if options.search_text.len() < 1 {
        print_message(MessageType::ERROR, "Param search_text cannot be empty.");
        return;
    }
    if options.ignore_case {
        print_message(MessageType::WARN, "Using ignore case mode, highlight print is disabled.");
    }
    let file_exts = match options.file_ext.as_str() {
        "" => vec![],
        ext => {
            ext.split(",").map(|s| s.trim()).filter(|s| s.len() > 0).map(|s| String::from(".") + s).collect()
        },
    };
    let total_file_count_cell = RefCell::new(0u64);
    let scaned_file_count_cell = RefCell::new(0u64);
    let matched_file_count_cell = RefCell::new(0u64);
    let total_dir_count_cell = RefCell::new(0u64);
    let scaned_dir_count_cell = RefCell::new(0u64);
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| {
        total_file_count_cell.replace_with(|&mut c| c + 1);
        let p_str = match p.to_str() {
            None => return,
            Some(s) => s,
        };
        if file_exts.len() > 0 {
            let mut file_ext_matches = false;
            for i in 0..file_exts.len() {
                if p_str.to_lowercase().ends_with(&file_exts[i]) {
                    file_ext_matches = true;
                    break;
                }
            }
            if ! file_ext_matches {
                return;
            }
        }
        let file_content = match read_file_content(p, options.parsed_large_text_file_size) {
            Err(err) => {
                if options.verbose {
                    print_lastline("");
                    print_message(MessageType::WARN, &format!("Read file {} failed: {}", p_str, err));
                }
                return;
            },
            Ok(c) => c,
        };
        scaned_file_count_cell.replace_with(|&mut c| c + 1);
        if match_lines(p_str, &file_content, &options) {
            matched_file_count_cell.replace_with(|&mut c| c + 1);
        }
    }, &|p| {
        total_dir_count_cell.replace_with(|&mut c| c + 1);
        match p.to_str() {
            None => (),
            Some(p_str) => {
                if (! options.scan_dot_git) && p_str.ends_with("/.git") {
                    if options.verbose {
                        print_lastline("");
                        print_message(MessageType::INFO, &format!("Skip .git dir: {}", p_str));
                    }
                    return false;
                }
                if options.skip_link_dir && is_symlink(p) {
                    if options.verbose {
                        print_lastline("");
                        print_message(MessageType::INFO, &format!("Skip link dir: {}", p_str));
                    }
                    return false;
                }
                scaned_dir_count_cell.replace_with(|&mut c| c + 1);
                print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10))
            },
        }
        true
    }).unwrap_or(());
    print_lastline("");
    print_message(MessageType::OK, &format!("Total dir count: {}, scaned dir count: {}",
                                    total_dir_count_cell.into_inner(),
                                    scaned_dir_count_cell.into_inner()));
    print_message(MessageType::OK, &format!("Total file count: {}, scaned file count: {}, matched file count: {}",
                                    total_file_count_cell.into_inner(),
                                    scaned_file_count_cell.into_inner(),
                                    matched_file_count_cell.into_inner()));
}


fn main() -> XResult<()> {
    let mut options = Options::new();
    options.parse_args();
    
    if options.version {
        print_version();
        return Ok(());
    }

    options.parsed_huge_file_size = parse_size(&options.huge_file_size)? as u64;
    options.parsed_large_text_file_size = parse_size(&options.large_text_file_size)? as u64;
    options.parsed_large_line_size = parse_size(&options.large_line_size)? as u64;

    let dir_path = match get_absolute_path(&options.dir) {
        None => {
            return Err(new_box_error(&format!("Cannot find dir: {}", options.dir)));
        },
        Some(path) => path,
    };
    let start = SystemTime::now();
    match options.target.as_str() {
        "huge" | "hugefile" => find_huge_files(&options, &dir_path),
        "text" => find_text_files(&options, &dir_path),
        unknown => {
            return Err(new_box_error(&format!("Unknown command: {}", unknown)));
        },
    }
    let cost_millis = SystemTime::now().duration_since(start.clone()).unwrap().as_millis();
    print_message(MessageType::OK, &format!("Finding finished, cost {} ms", cost_millis));
    Ok(())
}
