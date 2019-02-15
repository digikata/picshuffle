/*
 * Copyright (c) 2017, Alan Chen
 * See LICENCE file for BSD-2 terms
 */

use ignore::WalkBuilder;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use exif;

use std;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::path::{ Path, PathBuf };
use std::fs;

use chrono;
use chrono::Local;
use chrono::Datelike;
use chrono::TimeZone;
use std::convert::From;

use crate::options::Options;


pub fn hash_file(fname: &str, fast_hash: bool) -> Vec<u8>
{
    let mut h = Sha256::new();

    const HASHBUFSZ: usize = 4096 * 16;

    let mut f = File::open(fname).expect("open file");

    let mut buf = [0u8; HASHBUFSZ];
    if fast_hash {
        if let Ok(nbytes) = f.read(&mut buf) {
            h.input(&buf[0..nbytes]);
        }

    } else {
        while let Ok(nbytes) = f.read(&mut buf) {
            if nbytes == 0 { break; }
            h.input(&buf[0..nbytes]);
        }
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

pub type ScanPair = (String, Vec<u8>);
pub type ScanData = Vec<ScanPair>;

    // use std::cmp;
    // use std::ffi::OsString;

pub fn scan_path(opts: &Options) -> ScanData
{
    let dir = &opts.in_dir;

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
    vdat.sort_by(|a, b| a.cmp(b));

    let mut sd: ScanData = Vec::new();
    for p in &vdat {
        sd.push((p.clone(), hash_file(p, opts.fast_hash)));
    }
    sd
}

pub type CopyPair = (String, String); /// src, dst
pub type CopyList = Vec<CopyPair>;


/// Estimate file creation date. For now, just the year/month of file metadata
/// later maybe look at exif data if available
fn get_fs_create_date(path: &str) -> chrono::Date<Local>
{
    let md = fs::metadata(path).expect("can't access metadata");
    let crtime_sys = match md.created() {
        Ok(ct) => ct,
        Err(_) => {
            // try to fallback to modifiction time for systems without...
            md.modified().expect("Can't access creation or modification time")
        }
    };
    
    chrono::DateTime::<Local>::from(crtime_sys).date()
}

fn conv_field_datetime(fld: &exif::Field) -> Option<exif::DateTime>
{
    let val_asc = match fld.value {
        exif::Value::Ascii(ref asc) => asc,
        _ => return None
    };
    let dt = match exif::DateTime::from_ascii(&val_asc[..][0]) {
        Ok(dt) => dt,
        _ => return None,
    };
    Some(dt)
}

/// Estimate create date from exif data, None is retuned if the file is not an
/// exif
fn get_exif_create_date(path: &str) -> Option<chrono::Date<Local>>
{
    let file = fs::File::open(path).unwrap_or_else(|_| panic!("Couldn't open {}", path));
    let reader = match exif::Reader::new(&mut std::io::BufReader::new(&file)) {
        Ok(reader) => reader,
        Err(err) => {
            eprintln!("{:?}", err);
            return None;
        }
    };

    let mut ret: Option<chrono::Date<Local>> = None;
    for f in reader.fields() {
        match f.tag {
            exif::Tag::DateTime => {
                if let Some(dt) = conv_field_datetime(&f) {
                    // println!("DateTime: {}", dt);
                    // println!("YYYY/MM: {}/{}", dt.year, dt.month);
                    let year  = i32::from(dt.year);
                    let month = u32::from(dt.month);
                    let day   = u32::from(dt.day);
                    let date = Local.ymd(year, month, day);
                    if let Some(olddate) = ret {
                        if date < olddate {
                            ret = Some(date);
                        }
                    } else {
                        ret = Some(date);
                    }
                }
            },

            exif::Tag::DateTimeOriginal => {
                if let Some(dt) = conv_field_datetime(&f) {
                    // println!("DateTime: {}", dt);
                    // println!("YYYY/MM: {}/{}", dt.year, dt.month);
                    let year  = i32::from(dt.year);
                    let month = u32::from(dt.month);
                    let day   = u32::from(dt.day);
                    let date = Local.ymd(year, month, day);
                    if let Some(olddate) = ret {
                        if date < olddate {
                            ret = Some(date);
                        }
                    } else {
                        ret = Some(date);
                    }
                }
            },
            _ => {},
        }
    }
    ret
}


use std::collections::HashSet;

fn checked_add_to_copylist(clist: &mut CopyList, outpaths: &mut HashSet<String>, src: String, pdst: &PathBuf)
{
    let dst = String::from(pdst.to_str().unwrap());

    if outpaths.contains(&dst) {
        // different contents with same name
        // so, create a unique output file name
        let mut fidx = 1;

        let fname = match pdst.file_stem() {
            Some(_stem) => _stem.to_str().unwrap(),
            None => "",
        };
        let ext = match pdst.extension() {
            Some(_ext) => format!(".{}", _ext.to_str().unwrap()),
            None => String::new(),
        };

        let mut pdst = pdst.clone();
        pdst.pop();

        loop {
            let newname = format!("{}-{}{}", fname, fidx, ext);
            if !outpaths.contains(&newname) {
                pdst.push(newname);
                let dst = String::from(pdst.to_str().unwrap());
                clist.push((src, dst));
                break;
            }
            fidx += 1;
        }
    } else {
        // new file add to unique outpath and copy list
        outpaths.insert(dst.clone());
        clist.push((src, dst));
    }
}


/// create output file path:
/// outdir/YYYY/MM/srcfilename
fn make_outpath(opts: &Options, outdir: &str, srcpath: &str) -> PathBuf
{
    // create output path
    let mut pdst = PathBuf::from(outdir);
    let pbsrc = PathBuf::from(srcpath);

    // add year/month
    let crdate = if opts.ignore_exif {
        get_fs_create_date(&srcpath)
    } else {
        match get_exif_create_date(&srcpath) {
            Some(val) => val,
            None      => get_fs_create_date(&srcpath), // fallback to filesys date
        }
    };

    let stryear = format!("{}", crdate.year());
    pdst.push(stryear);
    let strmonth = format!("{}", crdate.month());
    pdst.push(strmonth);

    // finish creating dst name
    let src_fname = pbsrc.file_name().expect("bad file name").to_str().unwrap();
    pdst.push(src_fname);

    pdst
}


/// filter out repeated files based on their hash
/// and transform it to (src, dst) copy commands
pub fn filter_repeated(opts: &Options, scandata: &[ScanPair]) -> CopyList
{
    use std::collections::HashMap;

    let outdir = &opts.out_dir;

    let mut clist: CopyList = Vec::new();

    // track unique hash contents
    let mut hm = HashMap::<Vec<u8>, &str>::new();

    // track unique output names
    let mut outpaths = HashSet::new();

    for ent in scandata.iter() {
        let p = &ent.0; // source path
        let h = &ent.1; // hash of source

        let src = (*p).clone();

        if hm.contains_key(h) {
            // hash collision - check full hash of new file and existing file...

            if !opts.fast_hash {
                // no point doing more comparison the files were full hashes already
                continue;
            }
            let existing_src = &hm[h];
            let full_hash_old = hash_file(existing_src, false);
            let full_hash_new = hash_file(&src, false);

            if full_hash_new == full_hash_old {
                continue;
            }

            // the files are different after all, add to copy list
            let pdst = make_outpath(opts, outdir, p);
            checked_add_to_copylist(&mut clist, &mut outpaths, src, &pdst);
        } else {
            hm.insert(h.clone(), p);

            let pdst = make_outpath(opts, outdir, p);
            checked_add_to_copylist(&mut clist, &mut outpaths, src, &pdst);
        }
    }
    clist
}


/// execute a list of copy commands
pub fn exec_copies(cplist: &[CopyPair])
{
    use std::fs::copy;

    for cpair in cplist.iter() {
        let ppair = (Path::new(&cpair.0), Path::new(&cpair.1));

        let dst = ppair.1;
        if !dst.exists() {
            let parent_dir = dst.parent().expect("couldn't find parent out dir");
            if !parent_dir.exists() {
                if let Err(edir) = create_dir_all(parent_dir) {
                    eprintln!("error creating {:?}", edir);
                    return;
                }
            }
        }

        if let Err(e) = copy(ppair.0, ppair.1) {
            eprintln!("err {:?}", e);
        }
    }
}

/// generate a script of copy commands for unix systems
pub fn script_copies_unix(cplist: &[CopyPair])
{
    // track directories to create
    let mut create_dirs: HashSet<&str> = HashSet::new();
    for cpair in cplist.iter() {
        let ppair = (Path::new(&cpair.0), Path::new(&cpair.1));

        let dst = ppair.1;
        if !dst.exists() {
            let parent_dir = dst.parent().expect("couldn't find parent out dir");
            let parent_str = parent_dir.to_str().unwrap();
            if !parent_dir.exists() && !create_dirs.contains(parent_str) {
                create_dirs.insert(parent_str);
            }
        }
    }

    // generate script lines
    for dir in &create_dirs {
        println!("mkdir -p '{}'", dir);
    }
    for cpair in cplist.iter() {
        println!("cp '{}' '{}'", cpair.0, cpair.1);
    }
}


pub fn generic_dry_run(cplist: &[CopyPair])
{
    for cpair in cplist.iter() {
        println!("copy {} to {}", cpair.0, cpair.1);
    }
}


#[cfg(test)]
mod test
{
    #[cfg(test)]
    use crate::actions::CopyList;

    #[test]
    fn t_scan_path()
    {
        use crate::actions::scan_path;
        use crate::options;

        let mut opts =options::default();
        opts.in_dir = String::from("test/ref");

        let sinfo = scan_path(&opts);
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
    fn t_filter_repeated()
    {
        use crate::actions::scan_path;
        use crate::actions::filter_repeated;
        use crate::options;

        let mut opts =options::default();
        opts.in_dir = String::from("test/ref");
        opts.out_dir = String::from("test/out");

        let sinfo = scan_path(&opts);
        // for ent in sinfo.iter() {
        //     let p = &ent.0;
        //     let h = &ent.1;
        //     println!("{:?} {:?}", p, h);
        // }

        let filt = filter_repeated(&opts, &sinfo);
        for ent in filt.iter() {
            let src = &ent.0;
            let dst = &ent.1;
             println!("{:20} -> {}", src, dst);
        }
    }

    /// check that only the expected files exist
    #[cfg(test)]
    fn assert_file_iff(exp_list: &Vec<(&str, String)>, cplist: &CopyList)
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

            assert!(xp_files.contains_key(src), "Missing source {}", src);

            let val = xp_files.get_mut(src).unwrap();
            assert_eq!(val.0, dst, "Dest file mismatch...");
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
        use crate::actions::*;
        use crate::options;
        use std::fs;

        let mut opts = options::default();
        opts.ignore_exif = true;
        opts.in_dir = String::from("test/ref");
        opts.out_dir = String::from("test/out");

        let sinfo = scan_path(&opts);
        let cplist = filter_repeated(&opts, &sinfo);
        exec_copies(&cplist);

        // assume the ref creation yyyy/mmm date will be the test file date
        let created = get_fs_create_date("test/ref");
        let year = created.year();
        let mon  = created.month();

        let flist = vec![
            ("test/ref/a/1",   format!("test/out/{}/{}/1",  year, mon)),
            ("test/ref/a/10",  format!("test/out/{}/{}/10", year, mon)),
            // "test/ref/b/10", // b10 is a dup (expected to be filtered out)
            ("test/ref/b/foo", format!("test/out/{}/{}/foo", year, mon)),
        ];
        assert_file_iff(&flist, &cplist);

        // cleanup
        match fs::remove_dir_all(opts.out_dir) {
            Ok(_) => (),
            Err(_) => {
                assert!(false); // fail cleanup
            }
        }
    }

    #[test]
    fn t_deconflict_output_fname()
    {
        use crate::actions::*;
        use crate::options;

        let mut opts = options::default();
        opts.in_dir = String::from("test/ref2");
        opts.out_dir = String::from("test/out2");

        let sinfo = scan_path(&opts);
        let cplist = filter_repeated(&opts, &sinfo);

        exec_copies(&cplist);

        // assume the ref creation yyyy/mmm date will be the test file date
        let created = get_fs_create_date("test/ref2");
        let year = created.year();
        let mon  = created.month();

        let flist = vec![
            ("test/ref2/foo",   format!("test/out2/{}/{}/foo-1", year, mon)),
            ("test/ref2/b/foo", format!("test/out2/{}/{}/foo", year, mon)),
        ];
        assert_file_iff(&flist, &cplist);

        // cleanup
        use std::fs;
        match fs::remove_dir_all(opts.out_dir) {
            Ok(_) => (),
            Err(_) => {
                assert!(false); // fail cleanup
            }
        }
    }

    #[test]
    fn t_exif()
    {
        use crate::actions::*;
        use crate::options;

        let mut opts = options::default();
        opts.ignore_exif = false;
        opts.in_dir = String::from("test/ref3");
        opts.out_dir = String::from("test/out3");

        let sinfo = scan_path(&opts);
        let cplist = filter_repeated(&opts, &sinfo);
        exec_copies(&cplist);

        // assume the ref creation yyyy/mmm date will be the test file date
        let year = 2011;
        let mon  = 11;

        let flist = vec![
            ("test/ref3/8096135463_c7384a72c6_o.jpg",
                format!("test/out3/{}/{}/8096135463_c7384a72c6_o.jpg", year, mon)),
        ];
        assert_file_iff(&flist, &cplist);

        // cleanup
        use std::fs;
        match fs::remove_dir_all(opts.out_dir) {
            Ok(_) => (),
            Err(_) => {
                assert!(false); // fail cleanup
            }
        }
    }
}
