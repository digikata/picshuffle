/*
 * Copyright (c) 2017, Alan Chen
 * See LICENCE file for BSD-2 terms
 */

 //! spec ad manage utilities options

use clap::{ App, Arg };

/// store options selections parsed by args_to_opts()
pub struct Options {
    pub fast_hash: bool,
    pub dry_run: bool,
    pub ignore_exif: bool,
    pub out_script: bool,
    pub in_dir: String,
    pub out_dir: String,
}


pub fn default() -> Options
{
    Options {
        fast_hash: true,
        dry_run: false,
        ignore_exif: false,
        out_script: true,
        in_dir: String::new(),
        out_dir: String::new(),
    }
}



pub fn args_to_opts() -> Options
{
    let app = App::new("picshuffle")
        .version(env!("CARGO_PKG_VERSION"))
        .about("picshuffle is a utility to grab piles of photo files \n\
            and organize them into a destination directory")
        .arg(Arg::with_name("dir")
            .value_name("SCAN_DIR")
            .help("directory to scan")
            .required(true)
            )
        .arg(Arg::with_name("ignore_exif")
            .short("e")
            .help("Option to ignore exif dates from files")
            )
        .arg(Arg::with_name("full_hash")
            .short("f")
            .help("Option to perform fingerprint over full files")
            )
        .arg(Arg::with_name("out_script")
            .short("s")
            .help("Output organizing actions as a script")
            )
        .arg(Arg::with_name("outdir")
            .value_name("OUTPUT_DIR")
            .help("output directory (dry run if not supplied)")
            )
        ;
    let amats = app.get_matches();

    let mut opts = default();

    let dir = amats.value_of("dir").expect("missing value");
    opts.in_dir = String::from(dir);

    match amats.value_of("ignore_exif") {
        Some(_) => opts.ignore_exif = true,
        None => opts.ignore_exif = false,
    };

    opts.out_dir = match amats.value_of("outdir") {
        Some(od) => {
            println!("  output to: {}\n", od);
            String::from(od)
        },
        None => {
            println!("  dry run, no output\n");
            opts.dry_run = true;
            String::new()
        }
    };

    opts
}
