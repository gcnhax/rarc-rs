use std;
use std::io::{Read, Seek};
use nom;

pub fn read<P, O, E, R>(parser: P, mut rdr: R) -> Result<O, E>
where
    R: Read + Seek,
    E: From<std::io::Error> + From<nom::ErrorKind>,
    P: Fn(&[u8]) -> nom::IResult<&[u8], O>,
{
    let mut input: Vec<u8> = Vec::new();
    loop {
        match parser(&input) {
            nom::IResult::Done(_, parsed) => return Ok(parsed),
            nom::IResult::Error(err) => return Err(E::from(err)),
            nom::IResult::Incomplete(needed) => {
                let len = match needed {
                    nom::Needed::Unknown => input.len() + 1, // read one byte
                    nom::Needed::Size(len) => len,
                };

                (&mut rdr)
                    .take((len - input.len()) as u64)
                    .read_to_end(&mut input)?;
            }
        };
    }
}
