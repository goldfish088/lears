/*
CHUNK: {
    DATA_LENGTH      [ 0x?? 0x?? 0x?? 0x?? ] (constraint: <= 1<<31)
    CHUNK_TYPE       [ 0x?? 0x?? 0x?? 0x?? ] (constraint: each byte either in [0x41, 0x5a] or [0x61, 0x7a])
    CHUNK_DATA       [ 0x??    ...    0x?? ] (constraint: length == DATA_LENGTH)
    CRC              [ 0x?? 0x?? 0x?? 0x?? ] (constraint: checksum via CRC algo on CHUNK_TYPE + CHUNK_DATA)
}
*/

use std::fmt::{self, Display, Formatter};

use crate::chunk_type::ChunkType;

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Box<[u8]>,
    crc: u32,
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            length: self.length,
            chunk_type: self.chunk_type,
            data: self.data.clone(),
            crc: self.crc,
        }
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let data = data.into_boxed_slice();
        let crc = Self::compute_crc(&chunk_type, &data);
        Chunk {
            chunk_type,
            length: data.len() as u32,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn compute_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        let mut crc = 0xffffffff;

        // original: polynomial
        // (2^32 is omitted) x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x + 1
        // In binary form: 0000 0100 1100 0001 0001 1101 1011 0111
        // as hex: 0x4c11db7

        // However, For the purpose of separating into bytes and ordering, the least significant bit of the 32-bit
        // CRC is defined to be the coefficient of the x31 term, hence we "reflect the polynomial"
        // Reflected hex: 0xedb88320
        let poly: u32 = 0xedb88320;

        for b in chunk_type.bytes().iter().chain(data) {
            crc ^= *b as u32;

            for _ in 1..=8 {
                if crc & 1 == 0 {
                    crc >>= 1;
                } else {
                    crc = (crc >> 1) ^ poly;
                }
            }
        }

        crc ^ 0xffffffff
    }

    fn validate_crc(chunk_type: &ChunkType, data: &[u8], crc_bytes: [u8; 4]) -> bool {
        Self::compute_crc(chunk_type, data) == u32::from_be_bytes(crc_bytes)
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> crate::Result<String> {
        Ok(String::from_utf8(self.data.to_vec())?)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = crate::Error;
    fn try_from(raw_bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut length = [0_u8; 4];
        let mut iter = raw_bytes.iter();

        // length
        for be_byte in &mut length {
            *be_byte = *iter
                .next()
                .ok_or::<crate::Error>("Expected byte for length field".into())?;
        }

        let length = u32::from_be_bytes(length);
        if length > 1 << 31 {
            return Err("Length is larger than 2^31".into());
        }

        // chunk type
        let mut chunk_type = [0_u8; 4];
        for be_byte in &mut chunk_type {
            *be_byte = *iter
                .next()
                .ok_or::<crate::Error>("Expected byte for chunk type field".into())?
        }

        let chunk_type = ChunkType::try_from(chunk_type)?;

        if iter.len() > (length as usize) + 4 {
            return Err("More bytes than expected when combining data and crc fields".into());
        }

        let mut data = vec![0_u8; length as usize];

        for be_byte in &mut data {
            *be_byte = *iter
                .next()
                .ok_or::<crate::Error>("Expected byte for chunk data field".into())?
        }

        let mut crc = [0_u8; 4];
        for be_byte in &mut crc {
            *be_byte = *iter
                .next()
                .ok_or::<crate::Error>("Expected byte for crc field".into())?
        }

        if !Self::validate_crc(&chunk_type, data.as_slice(), crc) {
            Err("Validation failed for crc field".into())
        } else {
            let crc = u32::from_be_bytes(crc);
            let data = data.into_boxed_slice();
            Ok(Chunk {
                length,
                chunk_type,
                data,
                crc,
            })
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk: {{")?;
        writeln!(f, "\tLength: {}", self.length)?;
        writeln!(f, "\tType: {}", self.chunk_type)?;
        write!(f, "\tData: [")?;

        for b in &self.data {
            write!(f, " {:02x}", b)?;
        }

        writeln!(f, " ]")?;
        writeln!(f, "\tCRC-32: 0x{:x}", self.crc)?;
        write!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
