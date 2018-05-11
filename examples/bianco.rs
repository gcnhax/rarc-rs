extern crate rarc;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Seek, SeekFrom};

use rarc::Rarc;

fn main() {
    let file = File::open(Path::new("data/bianco0.rarc")).expect("file not found");
    let mut reader = BufReader::new(file);

    let mut rarc = Rarc::new(&mut reader).expect("couldn't open rarc file");

    // println!("{:?}", rarc);

    rarc::vfs::dump_tree(&rarc.fs.root);
}
