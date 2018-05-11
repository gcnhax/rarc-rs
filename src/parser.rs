use nom::{IResult, be_u16, be_u32};

use {Entry, Header, Node};

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    do_parse!(
        input,
        tag!("RARC") >>
        file_size: be_u32 >>
        tag!([0x00, 0x00, 0x00, 0x20]) >> // header length (always 0x20, this is just a validity assert)
        data_offset: be_u32 >>
        data_length: be_u32 >> // unknown
        take!(4) >> // data_length again?
        take!(8) >> // u32 unknown[2] = {0, 0};

        n_nodes: be_u32 >> nodes_offset: be_u32 >> n_entries: be_u32
            >> entries_offset: be_u32 >> strings_size: be_u32 >> strings_offset: be_u32
            >> n_files: be_u16 >> take!(2) >> take!(4) >> (Header {
            file_size: file_size,
            data_offset: data_offset + 0x20,
            data_length: data_length,

            n_nodes: n_nodes,
            nodes_offset: nodes_offset + 0x20,

            n_entries: n_entries,
            entries_offset: entries_offset + 0x20,

            strings_size: strings_size,
            strings_offset: strings_offset + 0x20,

            n_files: n_files,
        })
    )
}

pub fn parse_node(input: &[u8]) -> IResult<&[u8], Node> {
    do_parse!(
        input,
        id: take_str!(4) >> filename_offset: be_u32 >> filename_hash: be_u16 >> n_entries: be_u16
            >> entry_start_id: be_u32 >> (Node {
            id: String::from(id),
            name: None,
            filename_offset: filename_offset,
            filename_hash: filename_hash,
            n_entries: n_entries,
            entry_start_id: entry_start_id,
        })
    )
}

pub fn parse_entry(input: &[u8]) -> IResult<&[u8], Entry> {
    do_parse!(
        input,
        idx: be_u16 >>
        hash: be_u16 >>
        entry_type: be_u16 >>
        name_offset: be_u16 >>
        data_offset_or_node_index: be_u32 >>
        file_data_length: be_u32 >>
        take!(4) >> // unknown, always 0

        (
            match entry_type {
                0x200 => Entry::Folder {
                    name_offset: name_offset,
                    hash: hash,
                    name: None,

                    folder_node_idx: data_offset_or_node_index,
                },
                0x1100 => Entry::File {
                    idx: idx,
                    name_offset: name_offset,
                    hash: hash,
                    name: None,

                    data_offset: data_offset_or_node_index,
                    data_length: file_data_length,
                },
                _ => panic!("unsupported entry type!"),
            }
        )
    )
}

#[cfg(test)]
mod test {
    use super::*;

    static HANDCRAFTED_RARC_HEADER: &'static [u8] = &[
        0x52, 0x41, 0x52, 0x43, // RARC

        0x13, 0x37, 0x13, 0x37, // file_size
        0x00, 0x00, 0x00, 0x20, // header length

        0x55, 0x55, 0x55, 0x35, // offset to the file data - 0x20
        0x00, 0x00, 0x67, 0x76, // data length
        0x00, 0x00, 0x67, 0x76, // data length (again)

        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,

        0x00, 0x00, 0x00, 0x70, // n_nodes
        0x33, 0x33, 0x33, 0x13, // nodes_offset - 0x20

        0x00, 0x00, 0x00, 0xff, // n_entries
        0x53, 0x35, 0x33, 0x56, // entries_offset - 0x20

        0x00, 0x00, 0xff, 0xff, // strings_size
        0x32, 0x54, 0x73, 0x62, // strings_offset - 0x20

        0x15, 0x32, // number of files

        0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    /// Check that we can parse the Bianco Hills 0 szs header successfully
    #[test]
    fn test_parse_bianco_header() {
        let data = include_bytes!("../data/bianco0.rarc");

        let parse_result = parse_header(data);

        assert!(parse_result.is_done());

        if let IResult::Done(_, ref header) = parse_result {
            println!("{:?}", header);
        }
    }

    /// Check that a handcrafted header parses properly
    #[test]
    fn test_parse_header() {
        let parse_result = parse_header(HANDCRAFTED_RARC_HEADER);

        assert!(parse_result.is_done());
        assert_eq!(
            parse_result.unwrap().1,
            Header {
                file_size: 0x13371337,
                data_offset: 0x55555555,
                data_length: 0x6776,

                n_nodes: 0x70,
                nodes_offset: 0x33333333,

                n_entries: 0xff,
                entries_offset: 0x53353376,

                strings_size: 0xffff,
                strings_offset: 0x32547382,

                n_files: 0x1532,
            }
        );
    }

    /// Check that a handcrafted header inverts back to the input when `.write()`ing it
    #[test]
    fn test_header_invertibility() {
        let parse_result = parse_header(HANDCRAFTED_RARC_HEADER);

        assert!(parse_result.is_done());

        let header = parse_result.unwrap().1;

        let mut new_header_data: Vec<u8> = vec![];
        header.write(&mut new_header_data);

        assert_eq!(&new_header_data[..], &HANDCRAFTED_RARC_HEADER[..]);
    }
}
