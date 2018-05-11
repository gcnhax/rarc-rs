//! A crate for manipulating files in the Nintendo RARC archive format.

#[macro_use]
extern crate nom;
extern crate byteorder;
extern crate encoding;

#[cfg(test)]
#[macro_use] extern crate pretty_assertions;

mod error;
mod parse_read;
mod parser;
pub mod vfs;

use std::io::{Read, BufRead, Write, Seek, SeekFrom, Cursor};
use std::io;
use std::ops::Range;
use byteorder::{WriteBytesExt, BE};
use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_31J; // shift_jis

pub use error::Error;

/// A Nintendo RARC archive.
#[derive(Debug)]
pub struct Rarc<R> where R: Read + Seek {
    header: Header,
    nodes: Vec<Node>,
    entries: Vec<Entry>,
    string_table: Vec<u8>,
    reader: R,

    /// The filesystem contained in this archive.
    pub fs: vfs::Fs,
}

impl<R> Rarc<R> where R: Read + BufRead + Seek {
    /// Reads an archive from a reader, parsing metadata and constructing a virtual filesystem.
    pub fn new(mut rdr: R) -> Result<Rarc<R>, Error> {
        let header = Header::read(&mut rdr)?;

        if header.n_nodes == 0 {
            return Err(Error::NoNodes);
        }

        // read the string table
        let mut string_table = Vec::with_capacity(header.strings_size as usize);
        rdr.seek(SeekFrom::Start(header.strings_offset as u64))?;
        (&mut rdr).take(header.strings_size as u64).read_to_end(&mut string_table)?;

        // seek to the start of the node table
        rdr.seek(SeekFrom::Start(header.nodes_offset as u64))?;
        // read the nodes
        let mut nodes = vec![];
        for _ in 0..header.n_nodes {
            let mut node = Node::read(&mut rdr)?;
            node.read_name(&string_table)?;
            nodes.push(node);
        }

        if nodes[0].id != "ROOT".to_owned() {
            return Err(Error::NoRootNode);
        }

        // seek to the start of the entry table
        rdr.seek(SeekFrom::Start(header.entries_offset as u64))?;
        // read the entries
        let mut entries = vec![];
        for _ in 0..header.n_entries {
            let mut entry = Entry::read(&mut rdr)?;
            entry.read_name(&string_table)?;
            entries.push(entry);
        }

        let mut fs = vfs::Fs::new(vfs::Dir::new(nodes[0].name().unwrap()));

        fn node_to_dir(nodes: &Vec<Node>, entries: &Vec<Entry>, node: &Node, dir: &mut vfs::Dir) {
            for entry in &entries[node.entry_range()] {
                if entry.filename_offset() != 0 && entry.filename_offset() != 2 {
                    let fsnode = match *entry {
                        Entry::File {data_offset, data_length, ..} => {
                            let bounds = (data_offset as usize, data_length as usize);
                            vfs::Node::File(vfs::File::new(entry.name().unwrap(), bounds))
                        },
                        Entry::Folder {folder_node_idx, ..} => {
                            let mut subdir = vfs::Dir::new(entry.name().unwrap());
                            let node = &nodes[folder_node_idx as usize];
                            node_to_dir(nodes, entries, node, &mut subdir);

                            vfs::Node::Dir(subdir)
                        }
                    };

                    dir.add(fsnode);
                }
            }
        }

        node_to_dir(&nodes, &entries, &nodes[0], &mut fs.root);

        Ok(Rarc {
            header: header,
            nodes: nodes,
            entries: entries,
            string_table: string_table,

            reader: rdr,
            fs: fs,
        })
    }
}

/// The RARC file header and info block.
#[derive(Debug, PartialEq)]
pub struct Header {
    pub file_size: u32,
    pub data_offset: u32,
    pub data_length: u32,

    pub n_nodes: u32,
    pub nodes_offset: u32,

    pub n_entries: u32,
    pub entries_offset: u32,

    pub strings_size: u32,
    pub strings_offset: u32,

    pub n_files: u16,
}

impl Header {
    /// Parses a `Header` from a reader.
    pub fn read<R>(rdr: R) -> Result<Header, Error> where R: Read + Seek {
        parse_read::read(parser::parse_header, rdr)
    }

    /// Writes this header to a writer.
    pub fn write<W>(&self, mut wtr: W) -> Result<(), io::Error> where W: Write {
        wtr.write_all(b"RARC")?;
        wtr.write_u32::<BE>(self.file_size)?;
        wtr.write_u32::<BE>(0x20)?;
        wtr.write_u32::<BE>(self.data_offset - 0x20)?;
        wtr.write_u32::<BE>(self.data_length)?;
        wtr.write_u32::<BE>(self.data_length)?; // intentional dupe

        wtr.write_u32::<BE>(0)?; // unknown
        wtr.write_u32::<BE>(0)?; // unknown

        wtr.write_u32::<BE>(self.n_nodes)?;
        wtr.write_u32::<BE>(self.nodes_offset - 0x20)?;

        wtr.write_u32::<BE>(self.n_entries)?;
        wtr.write_u32::<BE>(self.entries_offset - 0x20)?;

        wtr.write_u32::<BE>(self.strings_size)?;
        wtr.write_u32::<BE>(self.strings_offset - 0x20)?;

        wtr.write_u16::<BE>(self.n_files)?;

        wtr.write_u16::<BE>(0)?;
        wtr.write_u32::<BE>(0)?;

        Ok(())
    }
}

/// A RARC directory node.
#[derive(Debug, PartialEq)]
pub struct Node {
    id: String,
    name: Option<String>,
    filename_offset: u32,
    filename_hash: u16,

    entry_start_id: u32,
    n_entries: u16,
}

impl Node {
    /// Parses a `Node` from a reader.
    pub fn read<R>(rdr: R) -> Result<Node, Error> where R: Read + Seek {
        parse_read::read(parser::parse_node, rdr)
    }

    /// Reads the name of this node from the string table.
    pub fn read_name(&mut self, table: &Vec<u8>) -> Result<(), Error> {
        let mut rdr = Cursor::new(&table);

        rdr.seek(SeekFrom::Start(self.filename_offset as u64))?;
        let mut str_buf: Vec<u8> = vec![];
        rdr.read_until(0x00, &mut str_buf)?; // null-terminated
        str_buf.pop(); // remove the null terminator before decoding as shift_jis

        self.name = Some(WINDOWS_31J.decode(&str_buf, DecoderTrap::Strict)
            .map_err(|e| Error::NameEncodingError(e.into_owned()))?);

        Ok(())
    }

    /// Returns the range of `Entry` indices pointed at by this node.
    pub fn entry_range(&self) -> Range<usize> {
        Range {
            start: self.entry_start_id as usize,
            end: self.entry_start_id as usize + self.n_entries as usize,
        }
    }

    /// Returns this node's filename. Returns `None` if the filename hasn't been read from the string table.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(String::as_str)
    }

    /// Writes this node to a writer.
    pub fn write<W>(&self, mut wtr: W) -> Result<(), io::Error> where W: Write {
        wtr.write_all(&self.id.as_bytes()[0..4])?; // truncate to make sure we don't botch alignment
        wtr.write_u32::<BE>(self.filename_offset)?;
        wtr.write_u16::<BE>(self.filename_hash)?;
        wtr.write_u16::<BE>(self.n_entries)?;
        wtr.write_u32::<BE>(self.entry_start_id)?;

        Ok(())
    }
}

/// A representation of a RARC 'Entry'. Can be either a file or a folder.
#[derive(Debug, PartialEq)]
pub enum Entry {
    /// RARC file metadata. Contains bounds for its data.
    File {
        idx: u16,
        hash: u16,
        name_offset: u16,
        name: Option<String>,

        data_offset: u32,
        data_length: u32,
    },

    /// RARC folder metadata. Points back at a node index containing pointers to the entries in the folder.
    Folder {
        hash: u16,
        name_offset: u16,
        name: Option<String>,

        folder_node_idx: u32,
    },
}

impl Entry {
    /// Parses an entry from a reader.
    pub fn read<R>(rdr: R) -> Result<Entry, Error> where R: Read + Seek {
        parse_read::read(parser::parse_entry, rdr)
    }

    /// Reads the name of this entry from the string table.
    pub fn read_name(&mut self, table: &Vec<u8>) -> Result<(), Error> {
        let mut rdr = Cursor::new(&table);

        rdr.seek(SeekFrom::Start(self.filename_offset() as u64))?;
        let mut str_buf: Vec<u8> = vec![];
        rdr.read_until(0x00, &mut str_buf)?; // null-terminated
        str_buf.pop(); // remove the null terminator before decoding as shift_jis

        let name_ = WINDOWS_31J.decode(&str_buf, DecoderTrap::Strict)
            .map_err(|e| Error::NameEncodingError(e.into_owned()))?;

        match *self {
            Entry::File {ref mut name, ..} => *name = Some(name_),
            Entry::Folder {ref mut name, ..} => *name = Some(name_),
        }

        Ok(())
    }

    /// Returns this entry's filename. Returns `None` if the filename hasn't been read from the string table.
    pub fn name(&self) -> Option<&str> {
        let name = match *self {
            Entry::File {ref name, ..} => name,
            Entry::Folder {ref name, ..} => name,
        };

        name.as_ref().map(String::as_str)
    }

    /// Returns the offset into the string table of this entry's filename.
    pub fn filename_offset(&self) -> u16 {
        match *self {
            Entry::File {name_offset, ..} => name_offset,
            Entry::Folder {name_offset, ..} => name_offset,
        }
    }
}

/// Compute the hash of a file or directory name, according to the algorithm RARC uses.
fn filename_hash(filename: &str) -> u16 {
    let mut hash: u16 = 0;

    for chr in filename.chars() {
        hash *= 3;
        hash += chr as u16;
    }

    hash
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_header_from_file() {
        use std::fs::File;
        use std::path::Path;
        use std::io::{Read, BufReader, Seek, SeekFrom, Cursor};

        let mut file = File::open(Path::new("data/bianco0.rarc")).expect("file not found");
        let mut reader = BufReader::new(file);

        let header = Header::read(&mut reader).expect("could not parse header");

        assert_eq!(reader.seek(SeekFrom::Current(0)).unwrap(), 0x40);
        assert_eq!(header, Header { file_size: 5600608, data_offset: 27200, data_length: 5573408, n_nodes: 48, nodes_offset: 64, n_entries: 766, entries_offset: 832, strings_size: 11040, strings_offset: 16160, n_files: 766 });
    }
}
