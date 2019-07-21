extern crate argparse;
extern crate rust_util;

use std::{
    path::Path,
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

// ---------------------------------------------------------------------------------------------------------
fn check_path(path: &Path) {
    //println!("-------------- {:?} {}", path, path.is_dir());
}

fn list_dir<F>(dir: &Path, fnc: &F) -> XResult<()> where F: Fn(&Path) -> () {
    list_dir_with_depth_check(&mut 0u32, dir, fnc)
}

fn list_dir_with_depth_check<F>(depth: &mut u32, dir: &Path, fnc: &F) -> XResult<()> where F: Fn(&Path) -> () {
    if *depth > 100u32 {
        // TODO error: return Err(Box::new("depth"));
    }
    for e in dir.read_dir()? {
        let ee = e?;
        println!("{}:{} {:?} {}", depth, "    ".repeat(*depth as usize), ee, ee.path().as_path().is_file());
        if ee.path().as_path().is_dir() {
            *depth += 1;
            match list_dir_with_depth_check(depth, ee.path().as_path(), fnc) {
                Err(_) => (), // TODO ...
                Ok(_) => (),
            }
            *depth -= 1;
        }
        fnc(ee.path().as_path());
    }
    Ok(())
}

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

    //list_dir(get_absolute_path("~").unwrap().as_path(), &check_path).ok();
    list_dir(get_absolute_path("~").unwrap().as_path(), &|_| {}).ok();

    println!("Hello, world!");
}
