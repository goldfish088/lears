#![allow(unused_variables)]

use std::fmt::{Debug, Display, Error, Formatter};
use std::str::FromStr;

struct ChunkType {
    raw_bytes: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.raw_bytes
    }

    fn is_byte_valid(byte: u8) -> bool {
        (b'a' <= byte && byte <= b'z') || (b'A' <= byte && byte <= b'Z')
    }

    fn is_valid(&self) -> bool {
        // Only need to check that the reserved bit is unset,
        // as the ascii character validation across all 4 bytes is
        // done at construction
        self.raw_bytes[2] & 0x20 == 0
    }

    fn is_critical(&self) -> bool {
        self.raw_bytes[0] & 0x20 == 0
    }

    fn is_public(&self) -> bool {
        self.raw_bytes[1] & 0x20 == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        self.raw_bytes[2] & 0x20 == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        self.raw_bytes[3] & 0x20 != 0
    }
}

impl PartialEq<ChunkType> for ChunkType {
    fn eq(&self, other: &ChunkType) -> bool {
        self.raw_bytes[0] == other.raw_bytes[0]
            && self.raw_bytes[1] == other.raw_bytes[1]
            && self.raw_bytes[2] == other.raw_bytes[2]
            && self.raw_bytes[3] == other.raw_bytes[3]
    }
}

impl Eq for ChunkType {}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = crate::Error;
    fn try_from(raw_bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let mut bytes: [u8; 4] = [0, 0, 0, 0];

        let mut pos = 0;
        for byte in raw_bytes {
            if !Self::is_byte_valid(byte) {
                return Err(format!("Invalid byte '{}' at position {}", byte, pos).into());
            }

            bytes[pos] = byte;
            pos += 1;
        }

        Ok(ChunkType { raw_bytes: bytes })
    }
}

impl FromStr for ChunkType {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            Err(format!("Invalid chunk type with length: {}", s.len()).into())
        } else {
            let mut raw_bytes: [u8; 4] = [0, 0, 0, 0];

            let mut pos = 0;
            for byte in s.as_bytes() {
                if !Self::is_byte_valid(*byte) {
                    return Err(format!("Invalid byte '{}' at position {}", byte, pos).into());
                }

                raw_bytes[pos] = *byte;
                pos += 1;
            }

            Ok(ChunkType { raw_bytes })
        }
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for b in self.raw_bytes {
            write!(f, "{}", char::from(b))?;
        }

        Ok(())
    }
}

impl Debug for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
