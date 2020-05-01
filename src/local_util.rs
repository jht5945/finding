use std::{
    cell::Cell,
    fs::File,
    path::Path,
    io::prelude::*,
};
use rust_util::{ XResult, new_box_error, };

#[derive(Debug)]
pub struct MatchLine {
    pub line_number: usize,
    pub line_string: String,
}

impl MatchLine {
    pub fn new(line_number: usize, line_string: String) -> MatchLine {
        MatchLine { line_number, line_string, }
    }
}

pub struct CountCell( Cell<u64> );

impl CountCell {
    pub fn new() -> CountCell {
        CountCell( Cell::new(0_u64) )
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.0.get()
    }

    #[inline]
    pub fn add(&self, i: u64) {
        self.0.set(self.0.get() + i);
    }

    #[inline]
    pub fn add_one(&self) {
        self.add(1);
    }
}

pub fn read_file_content<P: AsRef<Path>>(p: P, len_of_large_file: u64) -> XResult<String> {
    let file = p.as_ref();
    if !file.exists() {
        return Err(new_box_error(&format!("File not exists: {:?}", file)));
    }
    if !file.is_file() {
        return Err(new_box_error(&format!("File is not a file: {:?}", file)));
    }
    let file_len = file.metadata()?.len();
    if file_len >= len_of_large_file {
        return Err(new_box_error(&format!("File too large: {:?}, len: {}", file, file_len)));
    }
    let mut f = File::open(file)?;
    let mut content = String::with_capacity(file_len as usize);
    f.read_to_string(&mut content)?;

    Ok(content)
}
