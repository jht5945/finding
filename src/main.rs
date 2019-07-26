extern crate argparse;
extern crate term;
extern crate term_size;
extern crate rust_util;

use std::{
    fs::File,
    path::Path,
    io::prelude::*,
    time::SystemTime,
};

use argparse::{ArgumentParser, StoreTrue, Store};
use rust_util::*;

const VERSION: &str = "0.1";

fn print_version() {
    print!(r#"finding {}
Copyright (C) 2019 Hatter Jiang.
License MIT <https://opensource.org/licenses/MIT>

Written by Hatter Jiang
"#, VERSION);
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
    let huge_file_size_bytes = match parse_size(&options.huge_file_size) {
        Err(err) => {
            print_message(MessageType::ERROR, &format!("Parse size failed: {}", err));
            return;
        },
        Ok(bytes) => bytes as u64,
    };
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| {
        match p.metadata() {
            Err(_) => (),
            Ok(metadata) => {
                let len = metadata.len();
                if len >= huge_file_size_bytes {
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
            Some(p_str) => print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10)),
        }
        true
    }).unwrap_or(());
    print_lastline("");
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

fn match_lines(tag: &str, content: &String, ignore_case: bool, search_text: &String) {
    let lines = content.lines();
    let mut match_lines_vec = vec![];
    let mut l_no = 0usize;
    let the_search_text = &match ignore_case {
        true => search_text.to_lowercase(),
        false => search_text.to_string(),
    };
    for ln in lines {
        let matches = match ignore_case {
            true => ln.to_lowercase().contains(the_search_text),
            false => ln.contains(the_search_text),
        };
        if matches {
            match_lines_vec.push(MatchLine::new(l_no, ln.to_string()));
        }
        l_no += 1;
    }

    if match_lines_vec.len() > 0 {
        print_lastline("");
        print_message(MessageType::OK, &format!("Find in {}:", tag));
        for i in 0..match_lines_vec.len() {
            print!("{}: ", match_lines_vec[i].line_number + 1);
            match ignore_case {
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
    }
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
    let large_text_file_size_bytes = match parse_size(&options.large_text_file_size) {
        Err(err) => {
            print_message(MessageType::ERROR, &format!("Parse size failed: {}", err));
            return;
        },
        Ok(bytes) => bytes as u64,
    };
    walk_dir(&dir_path, &|_, _| (/* do not process error */), &|p| {
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
        let file_content = match read_file_content(p, large_text_file_size_bytes) {
            Err(_err) => {
                // TODO ... print_message(MessageType::ERROR, &format!("Read file {} failed: {}", p_str, err));
                return;
            },
            Ok(c) => c,
        };
        match_lines(p_str, &file_content, options.ignore_case, &options.search_text);
    }, &|p| {
        match p.to_str() {
            None => (),
            Some(p_str) => {
                if p_str.ends_with("/.git") {
                    return false;
                }
                print_lastline(&get_term_width_message(&format!("Scanning: {}", p_str), 10))
            },
        }
        true
    }).unwrap_or(());
    print_lastline("");
}

struct Options {
    version: bool,
    target: String,
    huge_file_size: String,
    large_text_file_size: String,
    dir: String,
    file_ext: String,
    ignore_case: bool,
    search_text: String,
}

fn main() {
    let mut options = Options {
        version: false,
        target: String::from("text"),
        huge_file_size: String::from("100M"),
        large_text_file_size: String::from("10M"),
        file_ext: String::new(),
        ignore_case: false,
        dir: String::from("."),
        search_text: String::new(),
    };
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("finding - command line find tool.");
        ap.refer(&mut options.target).add_option(&["-t", "--target"], Store, "Target, text, huge[file], default text");
        ap.refer(&mut options.dir).add_option(&["-d", "--dir"], Store, "Target directory, default current dir(.)");
        ap.refer(&mut options.huge_file_size).add_option(&["--huge-file"], Store, "Huge file size, default 100M");
        ap.refer(&mut options.large_text_file_size).add_option(&["--large-text-file"], Store, "Large text file, default 10M");
        ap.refer(&mut options.file_ext).add_option(&["-f", "--file-ext"], Store, "File ext, default all");
        ap.refer(&mut options.ignore_case).add_option(&["-i", "--ignore-case"], StoreTrue, "Ignore case, default false");
        ap.refer(&mut options.version).add_option(&["-v", "--version"], StoreTrue, "Print version");
        ap.refer(&mut options.search_text).add_argument("SEARCH TEXT", Store, "Search text");
        ap.parse_args_or_exit();
    }
    
    if options.version {
        print_version();
        return;
    }

    let dir_path = match get_absolute_path(&options.dir) {
        None => {
            print_message(MessageType::ERROR, &format!("Cannot find dir: {}", options.dir));
            return;
        },
        Some(path) => path,
    };
    let start = SystemTime::now();
    match options.target.as_str() {
        "huge" | "hugefile" => find_huge_files(&options, &dir_path),
        "text" => find_text_files(&options, &dir_path),
        unknown => {
            print_message(MessageType::ERROR, &format!("Unknown command: {}", unknown));
            return;
        },
    }
    let cost_millis = SystemTime::now().duration_since(start.clone()).unwrap().as_millis();
    print_message(MessageType::OK, &format!("Finding finished, cost {} ms", cost_millis));
}
