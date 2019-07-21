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

// ---------------------------------------------------------------------------------------------------------

fn main() {
    let mut version = false;
    let mut target = String::from("text");
    let mut huge_file_size = String::from("100M");
    let mut dir = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("finding - command line find tool.");
        ap.refer(&mut target).add_option(&["-t", "--target"], Store, "Target, text, huge[file], default text");
        ap.refer(&mut huge_file_size).add_option(&["--huge-file"], Store, "Huge file size, default 100M");
        ap.refer(&mut version).add_option(&["-v", "--version"], StoreTrue, "Print version");
        ap.refer(&mut dir).add_argument("DIR", Store, "Dir name, default current dir(.)");
        ap.parse_args_or_exit();
    }
    
    if version {
        print_version();
        return;
    }

    // --------------------------------------------------------------------------------------------------------
    println!("{:?}", get_home_path());
    println!("{:?}", get_absolute_path("."));
    println!("{:?}", get_absolute_path("../"));
    println!("{:?}", get_absolute_path("~"));
    println!("{:?}", get_absolute_path("~/.jssp"));
    println!("{:?}", get_absolute_path("~/.jsspx"));


    walk_dir(get_home_path().unwrap().as_path(), &|_, _| {}, &|p| {println!("{:?}",p)}, &|_| false).unwrap();

    println!("Hello, world!");
}
