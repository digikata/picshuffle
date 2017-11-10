

use ignore::WalkBuilder;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::path::{ Path, PathBuf };
use std::fs;

use chrono;
use chrono::Local;
use chrono::Datelike;
use std::convert::From;

pub fn hash_file(fname: &str) -> Vec<u8>
{
    let mut h = Sha256::new();


    let mut buf = [0u8; 4096];

    let mut f = File::open(fname).expect("open file");
    while let Ok(nbytes) = f.read(&mut buf) {
        if nbytes == 0 { break; }
        h.input(&buf[0..nbytes]);
    }

    let mut out = vec![0; h.output_bytes()];
    // println!("{:?}", h.output_bytes());
    // let mut out = Vec::with_capacity(h.output_bytes);
    // for _ in 0..h.output_bytes() {
    //     out.push(0);
    // }
    h.result(&mut out.as_mut_slice());
    out
}

pub type ScanData = Vec<(String, Vec<u8>)>;

pub fn scan_path(dir: &str) -> ScanData
{
    let walk = WalkBuilder::new(dir);

    let mut vdat: Vec<String> = Vec::new();

    let fsitr = walk.build();
    for res in fsitr {
        if let Ok(p) = res {
            if let Some(ft) = p.file_type() {
                if ft.is_file() {
                    let str = p.path().to_str().expect("invalid file path");
                    vdat.push(String::from(str))
                }
            }
        }
    }
    
    let mut sd: ScanData = Vec::new();
    for p in vdat.iter() {
        sd.push((p.clone(), hash_file(p)));
    }
    sd
}

pub type CopyPair = (String, String); /// src, dst
pub type CopyList = Vec<CopyPair>;


/// Estimate file creation date. For now, just the year/month of file metadata
/// later maybe look at exif data if available
fn get_create_date(path: &str) -> chrono::Date<Local>
{
    let md = fs::metadata(path).expect("can't access metadata");
    let crtime_sys = match md.created() {
        Ok(ct) => ct,
        Err(_) => {
            // try to fallback to modifiction time for systems without...
            md.modified().expect("Can't access creation or modification time")
        }
    };
    let crdate = chrono::DateTime::<Local>::from(crtime_sys).date();
    crdate
}

/// filter out repeated files based on their hash
/// and transform it to (src, dst) paths
pub fn filter_repeated(scandata: &ScanData, outdir: &str) -> CopyList
{
    let mut clist: CopyList = Vec::new();

    use std::collections::HashMap;
    let mut hm = HashMap::new();
    for ent in scandata.iter() {
        let p = &ent.0;
        let h = &ent.1;
        if hm.contains_key(h) {
            continue;
        } else {
            hm.insert(h, p);
            let src = (*p).clone();

            // create output path
            let mut pdst = PathBuf::from(outdir);
            let psrc = PathBuf::from(p);

            // add year/month
            let crdate = get_create_date(&src);
            let stryear = format!("{}", crdate.year());
            pdst.push(stryear);
            let strmonth = format!("{}", crdate.month());
            pdst.push(strmonth);

            // add filename
            pdst.push(psrc.file_name().expect("bad file name"));
            let dst = String::from(pdst.to_str().unwrap());

            clist.push((src, dst));
        }
    }
    clist
}


pub fn exec_copies(cplist: &CopyList) {
    use std::fs::copy;

    for cpair in cplist.iter() {
        let ppair = (Path::new(&cpair.0), Path::new(&cpair.1));

        let dst = ppair.1;
        if !dst.exists() {
            let parent_dir = dst.parent().expect("couldn't find parent");
            if !parent_dir.exists() {
                if let Err(edir) = create_dir_all(parent_dir) {
                    println!("error creating {:?}", edir);
                    return;
                }
            }
        }

        if let Err(e) = copy(ppair.0, ppair.1) {
            println!("err {:?}", e);
        }
    }
}


#[cfg(test)]
mod test {

    #[cfg(test)]
    use actions::CopyList;

    #[test]
    fn t_scan_path() {
        use actions::scan_path;

        let refdir = "test/ref";
        let sinfo = scan_path(refdir);
        // for ent in sinfo.iter() {
        //     let p = &ent.0;
        //     let h = &ent.1;
        //     println!("{:?} {:?}", p, h);
        // }

        use std::collections::HashMap;
        let mut hm = HashMap::new();
        for ent in sinfo.iter() {
            let p = &ent.0;
            let h = &ent.1;
            //println!("{:?} {:?}", p, h);
            if hm.contains_key(h) {
                //println!("repeated {:?} {:?}", p, h);
            } else {
                hm.insert(h, p);
            }
        }
    }

    #[test]
    fn t_filter_repeated() {
        use actions::scan_path;
        use actions::filter_repeated;

        let refdir = "test/ref";
        let outdir = "test/out";

        let sinfo = scan_path(refdir);
        // for ent in sinfo.iter() {
        //     let p = &ent.0;
        //     let h = &ent.1;
        //     println!("{:?} {:?}", p, h);
        // }

        let filt = filter_repeated(&sinfo, outdir);
        for ent in filt.iter() {
            let src = &ent.0;
            let dst = &ent.1;
             println!("{:20} -> {}", src, dst);
        }
    }

    /// check that only the expected files exist
    #[cfg(test)]
    fn assert_file_iff(exp_list: &Vec<(&str, &str)>, cplist: &CopyList)
    {
        use std::collections::HashMap;

        // key: src path
        // val: (dst path, bool file seen)
        let mut xp_files: HashMap<&str, (&str, bool)> = exp_list.iter().map(
            |src_dst| { 
                (&src_dst.0[..], (&src_dst.1[..], false))
            }).collect();

        for ent in cplist.iter() {
            let src = &ent.0[..];
            let dst = &ent.1[..];
            // println!("{} {}", src, dst);

            assert!(xp_files.contains_key(src), "Unexpected source {}", src);

            let val = xp_files.get_mut(src).unwrap();
            assert_eq!(val.0, dst, "Bad destination");
            val.1 = true;
        }

        for (k,v) in xp_files {
            let found = v.1;
            assert_eq!(found, true, "List missing file {}", k);
            //println!("{} {}", k, v);
        }
    }


    #[test]
    fn t_exec_copy() {
        use actions::*;

        let refdir = "test/ref";
        let outdir = "test/out";

        let sinfo = scan_path(refdir);
        let cplist = filter_repeated(&sinfo, outdir);
        exec_copies(&cplist);

        let flist = vec![
            ("test/ref/a/1",  "test/out/2017/10/1"),
            ("test/ref/a/10", "test/out/2017/10/10"),
            // "test/ref/b/10", // b10 is a dup (expected to be filtered out)
            ("test/ref/b/foo", "test/out/2017/10/foo"),
        ];
        assert_file_iff(&flist, &cplist);

        // cleanup
        use std::fs;
        match fs::remove_dir_all(outdir) {
            Ok(_) => (),
            Err(_) => {
                assert!(false); // fail cleanup
            }
        }
    }

    #[test]
    fn t_deconflict_file_names() {
        use actions::*;

        let refdir = "test/ref2";
        let outdir = "test/out2";

        let sinfo = scan_path(refdir);
        let cplist = filter_repeated(&sinfo, outdir);
        exec_copies(&cplist);

        let flist = vec![
            ("test/ref2/foo",   "test/out2/2017/11/foo"),
            ("test/ref2/b/foo", "test/out2/2017/11/foo-1"),
        ];
        assert_file_iff(&flist, &cplist);

        // cleanup
        use std::fs;
        match fs::remove_dir_all(outdir) {
            Ok(_) => (),
            Err(_) => {
                assert!(false); // fail cleanup
            }
        }
    }    
}
