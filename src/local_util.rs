
use std:: {
    fs::File,
    path::Path,
    io::prelude::*,
};

use rust_util::*;

pub fn get_term_width() -> Option<usize> {
    match term_size::dimensions() {
        None => None,
        Some((w, _h)) => Some(w),
    }
}

// thanks https://blog.csdn.net/star_xiong/article/details/89401149
pub fn find_char_boundary(s: &str, index: usize) -> usize {
    if s.len() <= index {
        return index;
    }
    let mut new_index = index;
    while !s.is_char_boundary(new_index) {
        new_index += 1;
    }
    new_index
}

pub fn get_term_width_message(message: &str, left: usize) -> String {
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

pub fn read_file_content(file: &Path, large_file_len: u64) -> XResult<String> {
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
