use std::{fmt::Display};
use crc::{Crc, CRC_32_ISO_HDLC};

use crate::{chunk_type::{ChunkType}, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    const LENGTH_BYTES_LEN: usize = 4;
    const CHUNK_TYPE_BYTES_LEN: usize = 4;
    const CRC_LEN: usize = 4;
    const METADATA_BYTES_LEN: usize = Self::LENGTH_BYTES_LEN + Self::CHUNK_TYPE_BYTES_LEN + Self::CRC_LEN;

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = Self::crc_checksum(&chunk_type, &data);
        let length = data.len() as u32;
        Self {length: length, chunk_type: chunk_type, data: data, crc: crc}
    }
    
    pub fn length(&self) -> usize {
        self.length as usize
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone()).map_err(Into::into)
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

    pub fn crc_checksum(chunk_type: &ChunkType, data: &Vec<u8>) -> u32 {
        let bytes: Vec<_> = chunk_type
            .bytes()
            .iter()
            .chain(data.iter())
            .copied()
            .collect();

        Self::CRC32.checksum(&bytes)
    }

}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self>{
        if value.len() < Self::METADATA_BYTES_LEN {
            return Err(Box::from(ChunkError::InvalidChunkLength));
        }

        let length: usize = u32::from_be_bytes(value[0..4].try_into().unwrap()) as usize;
        if value.len() < length + Self::METADATA_BYTES_LEN {
            return Err(Box::from(ChunkError::InvalidChunkLength));
        }

        let chunk_type: [u8;4] = value[4..8].try_into()?;
        let chunk_type: ChunkType = chunk_type.try_into().unwrap();

        let data: Vec<u8> = value[8..8 + length].to_vec();
        let crc: u32 = u32::from_be_bytes(value[8+length..length+12].try_into().unwrap());

        let checksum = Self::crc_checksum(&chunk_type, &data);
        if crc != checksum {
            return Err(Box::from(ChunkError::InvalidCrc(crc, checksum)));
        }

        Ok(Self {
            length: length as u32,
            chunk_type: chunk_type,
            data: data,
            crc: crc,
        })

    }
}

#[derive(Debug)]
pub enum ChunkError {
    InvalidChunkLength,
    InvalidCrc(u32, u32),
}

impl std::error::Error for ChunkError {}

impl Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::InvalidChunkLength => {
                write!(f, "Invalid chunk length")
            }
            Self::InvalidCrc(expected, actual) => {
                write!(f, "Invalid crc {}, {}", expected, actual)
            }
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data_as_string().unwrap();
        write!(
            f,
            "Chunk {{ length: {}, chunk_type: {:?}, data: {}, crc: {} }}",
            self.length, self.chunk_type, data, self.crc
        )
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
        let data = "This is where your secret message will be!".as_bytes().to_vec();
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
        assert_eq!(chunk_data, chunk.as_bytes());
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
