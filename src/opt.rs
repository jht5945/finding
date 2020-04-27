use argparse::{ArgumentParser, StoreTrue, Store};
use rust_util::{ XResult, util_size::*, };

pub struct Options {
    pub version: bool,
    pub target: String,
    pub huge_file_size: String,
    pub parsed_huge_file_size: u64,
    pub large_text_file_size: String,
    pub parsed_large_text_file_size: u64,
    pub dir: String,
    pub file_ext: String,
    pub ignore_case: bool,
    pub filter_large_line: bool,
    pub large_line_size: String,
    pub parsed_large_line_size: u64,
    pub scan_dot_git: bool,
    pub skip_dot_dir: bool,
    pub skip_link_dir: bool,
    pub filter_file_name: String,
    pub filter_line_content: String,
    pub verbose: bool,
    pub search_text: String,
}

impl Options {
    pub fn new() -> Options {
        Options {
            version: false,
            target: String::from("text"),
            huge_file_size: String::from("100M"),
            parsed_huge_file_size: 0u64,
            large_text_file_size: String::from("10M"),
            parsed_large_text_file_size: 0u64,
            file_ext: String::new(),
            ignore_case: false,
            dir: String::from("."),
            filter_large_line: false,
            large_line_size: String::from("10KB"),
            parsed_large_line_size: 0u64,
            scan_dot_git: false,
            skip_dot_dir: false,
            skip_link_dir: false,
            filter_file_name: String::new(),
            filter_line_content: String::new(),
            verbose: false,
            search_text: String::new(),
        }
    }

    pub fn parse_args(&mut self) -> XResult<()> {
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("finding - command line find tool.");
            ap.refer(&mut self.target).add_option(&["-t", "--target"], Store, "Target, text, huge[file], default text");
            ap.refer(&mut self.dir).add_option(&["-d", "--dir"], Store, "Target directory, default current dir(.)");
            ap.refer(&mut self.huge_file_size).add_option(&["--huge-file"], Store, "Huge file size, default 100M");
            ap.refer(&mut self.large_text_file_size).add_option(&["--large-text-file"], Store, "Large text file, default 10M");
            ap.refer(&mut self.file_ext).add_option(&["-f", "--file-ext"], Store, "File ext, default all");
            ap.refer(&mut self.ignore_case).add_option(&["-i", "--ignore-case"], StoreTrue, "Ignore case, default false");
            ap.refer(&mut self.filter_large_line).add_option(&["--filter-large-line"], StoreTrue, "Filter large line");
            ap.refer(&mut self.large_line_size).add_option(&["--large-line-size"], Store, "Large line, default 10KB");
            ap.refer(&mut self.scan_dot_git).add_option(&["--scan-dot-git"], StoreTrue, "Scan dot git");
            ap.refer(&mut self.skip_dot_dir).add_option(&["--skip-dot-dir"], StoreTrue, "Skipt dot dir [Text Mode]");
            ap.refer(&mut self.skip_link_dir).add_option(&["--skip-link-dir"], StoreTrue, "Skip link dir");
            ap.refer(&mut self.filter_file_name).add_option(&["--filter-file-name"], Store, "Filter file name [Text Mode]");
            ap.refer(&mut self.filter_line_content).add_option(&["--filter-line-content"], Store, "Filter line content [Text Mode]");
            ap.refer(&mut self.version).add_option(&["-v", "--version"], StoreTrue, "Print version");
            ap.refer(&mut self.verbose).add_option(&["--verbose"], StoreTrue, "Verbose");
            ap.refer(&mut self.search_text).add_argument("SEARCH TEXT", Store, "Search text");
            ap.parse_args_or_exit();
        }
        
        self.parsed_huge_file_size = parse_size(&self.huge_file_size)? as u64;
        self.parsed_large_text_file_size = parse_size(&self.large_text_file_size)? as u64;
        self.parsed_large_line_size = parse_size(&self.large_line_size)? as u64;
        Ok(())
    }

    pub fn new_and_parse_args() -> XResult<Options> {
        let mut options = Options::new();
        options.parse_args()?;
        Ok(options)
    }
}