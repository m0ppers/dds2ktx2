use byteorder::{LittleEndian, WriteBytesExt};
use std::default;
use std::io::Error as IoError;
use std::io::Write;

#[derive(Debug)]
struct KTX2Header {
    identifier: [u8; 12],
    vk_format: u32,
    type_size: u32,
    pixel_width: u32,
    pixel_height: u32,
    pixel_depth: u32,
    layer_count: u32,
    face_count: u32,
    level_count: u32,
    supercompression_scheme: u32,

    // Index
    dfd_byte_offset: u32,
    dfd_byte_length: u32,
    kvd_byte_offset: u32,
    kvd_byte_length: u32,
    sgd_byte_offset: u64,
    sgd_byte_length: u64,
}

impl default::Default for KTX2Header {
    fn default() -> Self {
        KTX2Header {
            identifier: [
                0xAB, 'K' as u8, 'T' as u8, 'X' as u8, ' ' as u8, '2' as u8, '0' as u8, 0xBB,
                '\r' as u8, '\n' as u8, 0x1A, '\n' as u8,
            ],
            vk_format: 0,
            type_size: 0,
            pixel_width: 0,
            pixel_height: 0,
            pixel_depth: 0,
            layer_count: 0,
            face_count: 0,
            level_count: 0,
            supercompression_scheme: 0,
            dfd_byte_offset: 0,
            dfd_byte_length: 0,
            kvd_byte_offset: 0,
            kvd_byte_length: 0,
            sgd_byte_offset: 0,
            sgd_byte_length: 0,
        }
    }
}

impl KTX2Header {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        writer.write_all(&self.identifier)?;
        writer.write_u32::<LittleEndian>(self.vk_format)?;
        writer.write_u32::<LittleEndian>(self.type_size)?;
        writer.write_u32::<LittleEndian>(self.pixel_width)?;
        writer.write_u32::<LittleEndian>(self.pixel_height)?;
        writer.write_u32::<LittleEndian>(self.pixel_depth)?;
        writer.write_u32::<LittleEndian>(self.layer_count)?;
        writer.write_u32::<LittleEndian>(self.face_count)?;
        writer.write_u32::<LittleEndian>(self.level_count)?;
        writer.write_u32::<LittleEndian>(self.supercompression_scheme)?;
        writer.write_u32::<LittleEndian>(self.dfd_byte_offset)?;
        writer.write_u32::<LittleEndian>(self.dfd_byte_length)?;
        writer.write_u32::<LittleEndian>(self.kvd_byte_offset)?;
        writer.write_u32::<LittleEndian>(self.kvd_byte_length)?;
        writer.write_u64::<LittleEndian>(self.sgd_byte_offset)?;
        writer.write_u64::<LittleEndian>(self.sgd_byte_length)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct KTX2 {
    header: KTX2Header,
    levels: Vec<Vec<u8>>,
}

impl KTX2 {
    pub fn new() -> Self {
        KTX2 {
            ..default::Default::default()
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.header.pixel_width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.header.pixel_height = height;
        self
    }

    pub fn levels(mut self, levels: u32) -> Self {
        self.header.level_count = levels;
        self
    }

    pub fn vk_format(mut self, vk_format: u32) -> Self {
        self.header.vk_format = vk_format;
        self
    }

    pub fn face_count(mut self, face_count: u32) -> Self {
        self.header.face_count = face_count;
        self
    }

    pub fn add_level(mut self, level_data: &[u8]) -> Self {
        self.levels.push(level_data.to_vec());
        self
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), IoError> {
        self.header.write(writer)?;
        let mut offset =
            std::mem::size_of::<KTX2Header>() + 4 + (24 * self.header.level_count as usize); // +dfdTotalSize

        for level in self.levels.iter() {
            writer.write_u64::<LittleEndian>(offset as u64)?;
            writer.write_u64::<LittleEndian>(level.len() as u64)?;
            writer.write_u64::<LittleEndian>(level.len() as u64)?;
            offset = offset + level.len();
        }
        writer.write_u32::<LittleEndian>(0)?; // dfdTotalSize
        for level in self.levels.iter() {
            writer.write_all(level)?;
        }
        Ok(())
    }
}
