

use ignore::WalkBuilder;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;


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


pub type FileList = Vec<(String, String)>;

/// filter out repeated files based on their hash
/// and transform it to (src, dst) paths
pub fn filter_repeated(scandata: &ScanData, outdir: &str) -> FileList
{
    let mut filtered: FileList = Vec::new();

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
            pdst.push(psrc.file_name().expect("bad file name"));
            let dst = String::from(pdst.to_str().unwrap());

            filtered.push((src, dst));
        }
    }
    filtered
}

#[cfg(test)]
mod test {

    #[test]
    fn t_scan_path() {
        use scan::scan_path;

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
    fn filter_repeated() {
        use scan::scan_path;
        use scan::filter_repeated;

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
}
