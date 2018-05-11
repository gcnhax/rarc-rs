extern crate rarc;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader};

use rarc::Rarc;

fn main() {
    let file = File::open(Path::new("data/bianco0.rarc")).expect("file not found");
    let mut reader = BufReader::new(file);

    let rarc = Rarc::new(&mut reader).expect("couldn't open rarc file");

    rarc::vfs::dump_tree(&rarc.fs.root);
}
