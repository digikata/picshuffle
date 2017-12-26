

extern crate chrono;
extern crate clap;
extern crate crypto;
extern crate ignore;
extern crate exif;

use clap::{ App, Arg };


mod options;
mod actions;
use actions::scan_path;
use actions::filter_repeated;
use actions::exec_copies;

fn args_to_opts() -> options::Options 
{
    let app = App::new("picshuffle")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("dir")
            .value_name("SCAN_DIR")
            .help("directory to scan")
            .required(true)
            )
        .arg(Arg::with_name("use_exif")
            .short("e")
            .help("Option to extract exif dates from files")
            )
        .arg(Arg::with_name("full_hash")
            .short("f")
            .help("Option to perform fingerprint over full files")
            )
        .arg(Arg::with_name("outdir")
            .value_name("OUTPUT_DIR")
            .help("output directory (dry run if not supplied)")
            )
        ;
    let amats = app.get_matches();

    let mut opts = options::default();

    let dir = amats.value_of("dir").expect("missing value");
    opts.in_dir = String::from(dir);

    match amats.value_of("use_exif") {
        Some(_) => opts.use_exif = true,
        None => opts.use_exif = false,
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



fn main() {
    let opts = args_to_opts();

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
