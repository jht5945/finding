use std::{
    fs::File,
    path::Path,
    io::prelude::*,
};
use rust_util::{ XResult, new_box_error, };

pub fn read_file_content(file: &Path, large_file_len: u64) -> XResult<String> {
    if !file.exists() {
        return Err(new_box_error(&format!("File not exists: {:?}", file)));
    }
    if !file.is_file() {
        return Err(new_box_error(&format!("File is not a file: {:?}", file)));
    }
    let file_len = file.metadata()?.len();
    if file_len > large_file_len {
        return Err(new_box_error(&format!("File too large: {:?}, len: {}", file, file_len)));
    }
    let mut f = File::open(file)?;
    let mut content = String::with_capacity(file_len as usize);
    f.read_to_string(&mut content)?;

    Ok(content)
}
