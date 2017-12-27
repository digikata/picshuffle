/*
 * Copyright (c) 2017, Alan Chen
 * See LICENCE file for BSD-2 terms
 */

extern crate chrono;
extern crate clap;
extern crate crypto;
extern crate ignore;
extern crate exif;




mod options;
mod actions;
use actions::scan_path;
use actions::filter_repeated;
use actions::exec_copies;
use actions::script_copies_unix;



/// picshuffle is a utility to grab piles of photo files and organize them into a
/// destination directory
fn main() {
    let opts = options::args_to_opts();

    println!("scan {}", opts.in_dir);

    let cplist = filter_repeated(&opts, &scan_path(&opts));

    if opts.dry_run {
        for cpair in cplist.iter() {
            println!("copy {} to {}", cpair.0, cpair.1);
        }
    } else {
        exec_copies(&cplist);
    }
}
