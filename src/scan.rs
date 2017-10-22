

use ignore::WalkBuilder;

fn scan_path(dir: &str) {
    
    let walk = WalkBuilder::new(dir);

    let mut vdat: Vec<String> = Vec::new();

    let fsitr = walk.build();
    for res in fsitr {
        let dirstr = match res {
            Ok(p) => {
                if let Some(ft) = p.file_type() {
                    if ft.is_file() {
                        let str = p.path().to_str().expect("invalide file path");
                        String::from(str)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            },
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        vdat.push(dirstr);
    }
    
    for p in vdat.iter() {
        println!("{:?}", p);
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn scan_list() {
        use scan::scan_path;

        let refdir = "test/ref";
        scan_path(refdir);
    }
}
