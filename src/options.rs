

pub struct Options {
    pub fast_hash: bool,
    pub dry_run: bool,
    pub in_dir: String,
    pub out_dir: String,
}


pub fn default() -> Options 
{
    Options {
        fast_hash: true,
        dry_run: false,

        in_dir: String::new(),
        out_dir: String::new(),
    }
}