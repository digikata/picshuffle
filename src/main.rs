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
use actions::generic_dry_run;


/// picshuffle is a utility to grab piles of photo files and organize them into a
/// destination directory
fn main() {
    let opts = options::args_to_opts();

    eprintln!("scan {}", opts.in_dir);

    let scandat = scan_path(&opts);
    let cplist  = filter_repeated(&opts, &scandat);

    if opts.out_script {
        script_copies_unix(&cplist);
    } else if opts.dry_run {
        generic_dry_run(&cplist);
    } else {
        exec_copies(&cplist);
    }
}
