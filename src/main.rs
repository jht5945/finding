extern crate argparse;
extern crate rust_util;

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

fn main() {
    let mut version = false;
    let mut huge_file_size = String::from("100M");
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("finding - command line find tool.");
        ap.refer(&mut huge_file_size).add_option(&["--huge-file"], Store, "Huge file size, default 100M");
        ap.refer(&mut version).add_option(&["-v", "--version"], StoreTrue, "Print version");
        ap.parse_args_or_exit();
    }
    
    if version {
        print_version();
        return;
    }

    println!("Hello, world!");
}
