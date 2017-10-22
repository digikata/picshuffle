

extern crate ignore;
extern crate crypto;
extern crate clap;

use clap::{ App, Arg };

mod scan;
use scan::scan_path;


fn main() {
    let app = App::new("pcoalesce")
        .version("0.1")
        .arg(Arg::with_name("dir")
            .short("d")
            )
        ;

    let opts = app.get_matches();
    if let Some(dir) = opts.value_of("dir") {
        scan_path(dir);
    }
}
