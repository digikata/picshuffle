

extern crate ignore;
extern crate crypto;
extern crate clap;

use clap::{ App, Arg };

mod scan;
use scan::scan_path;
use scan::filter_repeated;

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


    let outdir = match opts.value_of("outdir") {
        Some(od) => {
            println!("  output to: {}\n", od);
            String::from(od)
        },
        None => {
            println!("  dry run, no output\n");
            String::new()
        }
    };

    let fdat = filter_repeated(&scan_path(dir), &outdir);
    for ent in fdat.iter() {
        println!("{:?}", ent.0);
    }

}
