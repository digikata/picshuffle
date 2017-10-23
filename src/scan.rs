

use ignore::WalkBuilder;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use std::fs::File;
use std::io::Read;

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


pub fn scan_path(dir: &str) {
    
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
    
    let mut hashes: Vec<(&str, Vec<u8>)> = Vec::new();
    for p in vdat.iter() {
        
        let h = hash_file(p);
        println!("{:?} {:?}", p, h);
        hashes.push((p, h));
    }
}


#[cfg(test)]
mod test {

    #[test]
    fn t_scan_path() {
        use scan::scan_path;

        let refdir = "test/ref";
        scan_path(refdir);
    }
}
