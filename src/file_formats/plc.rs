use byteorder::{ReadBytesExt, BE};
use std::io::{Error, ErrorKind, Read, Seek};

#[derive(Debug, Clone)]
pub struct PLCEntry {
    pub codes: [u32; 5],
}

#[derive(Debug, Clone)]
pub struct PLC {
    pub entries: Vec<PLCEntry>,
}

impl PLC {
    pub fn from_file<R: Seek + Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if &signature != b"SPLC" {
            return Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                "Invalid Magic",
            )));
        }

        let entry_size = reader.read_u16::<BE>()?;
        let num_entries = reader.read_u16::<BE>()?;

        if entry_size != 0x14 {
            return Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                "Invalid Entry Size",
            )));
        }

        let mut entries = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            entries.push(PLCEntry {
                codes: [
                    reader.read_u32::<BE>()?,
                    reader.read_u32::<BE>()?,
                    reader.read_u32::<BE>()?,
                    reader.read_u32::<BE>()?,
                    reader.read_u32::<BE>()?,
                ],
            })
        }

        Ok(Self { entries })
    }
}
