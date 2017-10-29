

extern crate chrono;
extern crate clap;
extern crate crypto;
extern crate ignore;

use clap::{ App, Arg };

mod actions;
use actions::scan_path;
use actions::filter_repeated;
use actions::exec_copies;


fn main() {
    let app = App::new("pcoalesce")
        .version("0.1")
        .arg(Arg::with_name("dir")
            .value_name("SCAN_DIR")
            .help("directory to scan")
            .required(true)
            )
        .arg(Arg::with_name("outdir")
            .value_name("OUTPUT_DIR")
            .help("output directory (dry run if not supplied)")
            )
        ;

    let opts = app.get_matches();

    let dir = opts.value_of("dir").expect("missing value");
    println!("scan {}", dir);

    let mut opt_dry_run = false;
    let outdir = match opts.value_of("outdir") {
        Some(od) => {
            println!("  output to: {}\n", od);
            String::from(od)
        },
        None => {
            println!("  dry run, no output\n");
            opt_dry_run = true;
            String::new()
        }
    };

    let cplist = filter_repeated(&scan_path(dir), &outdir);

    if opt_dry_run {
        for cpair in cplist.iter() {
            println!("copy {} to {}", cpair.0, cpair.1);
        }
    } else {
        exec_copies(&cplist);
    }
}
