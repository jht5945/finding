extern crate argparse;
extern crate term_size;
extern crate rust_util;

use std::{
    fs::File,
    path::Path,
    io::prelude::*,
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

fn get_term_width_message(message: &str, left: usize) -> String {
    match get_term_width() {
        None => message.to_string(),
        Some(w) => {
            let len = message.len();
            if w > len {
               return message.to_string();
            }
            let mut s = String::new();
            s.push_str(&message[0..w-10-5-left]);
            s.push_str("[...]");
            s.push_str(&message[len-10..]);
            s
        },
    }
}

fn find_huge_files(huge_file_size: &String, dir_path: &Path) {
    let huge_file_size_bytes = match parse_size(&huge_file_size) {
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

fn main() {
    let mut version = false;
    let mut target = String::from("text");
    let mut huge_file_size = String::from("100M");
    let mut large_text_file_size = String::from("10M");
    let mut dir = String::from(".");
    let mut search_text = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("finding - command line find tool.");
        ap.refer(&mut target).add_option(&["-t", "--target"], Store, "Target, text, huge[file], default text");
        ap.refer(&mut dir).add_option(&["-d", "--dir"], Store, "Target directory, default current dir(.)");
        ap.refer(&mut huge_file_size).add_option(&["--huge-file"], Store, "Huge file size, default 100M");
        ap.refer(&mut large_text_file_size).add_option(&["--large-text-file"], Store, "Large text file, default 10M");
        ap.refer(&mut version).add_option(&["-v", "--version"], StoreTrue, "Print version");
        ap.refer(&mut search_text).add_argument("SEARCH TEXT", Store, "Search text");
        ap.parse_args_or_exit();
    }
    
    if version {
        print_version();
        return;
    }

    let dir_path = match get_absolute_path(&dir) {
        None => {
            print_message(MessageType::ERROR, &format!("Cannot find dir: {}", dir));
            return;
        },
        Some(path) => path,
    };
    match target.as_str() {
        "huge" | "hugefile" => find_huge_files(&huge_file_size, &dir_path),
        unknown => print_message(MessageType::ERROR, &format!("Unknown command: {}", unknown)),
    }
}
