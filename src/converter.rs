use std::error::Error;
use std::io::{Read, Write};

use crate::ktx2::KTX2;

use ddsfile::Caps2;
use ddsfile::Dds;

pub fn dds_format2ktx2_format(format: ddsfile::DxgiFormat) -> Option<u32> {
    match format {
        ddsfile::DxgiFormat::R16G16B16A16_Float => Some(vk_sys::FORMAT_R16G16B16A16_SFLOAT),
        ddsfile::DxgiFormat::R11G11B10_Float => Some(vk_sys::FORMAT_B10G11R11_UFLOAT_PACK32),
        _ => None,
    }
}

pub fn pixel_size(format: u32) -> Option<u32> {
    match format {
        vk_sys::FORMAT_R16G16B16A16_SFLOAT => Some(8),
        vk_sys::FORMAT_B10G11R11_UFLOAT_PACK32 => Some(4),
        _ => None,
    }
}

pub fn convert<I: Read, O: Write>(reader: &mut I, writer: &mut O) -> Result<(), Box<dyn Error>> {
    let mut dds = Dds::read(reader)?;

    let is_cubemap = dds.header.caps2 & Caps2::CUBEMAP == Caps2::CUBEMAP;
    if !is_cubemap {
        panic!("DDS doesn't have cubemap flag");
    }
    let format = match dds.get_dxgi_format() {
        Some(dxgi) => dxgi,
        None => panic!("DDS not in dxgi format"),
    };

    let vk_format = match dds_format2ktx2_format(format) {
        Some(vk_format) => vk_format,
        None => panic!("Unsupported format. Only R16G16B16A16_Float supported"),
    };

    let width_base = (dds.get_width() as f32).log2();
    let height_base = (dds.get_height() as f32).log2();
    if width_base.floor() != width_base.ceil() || height_base.floor() != height_base.ceil() {
        panic!(
            "Texture is not power of 2! {}x{}",
            dds.get_width(),
            dds.get_height()
        );
    }

    let levels = dds.get_num_mipmap_levels();
    let mut ktx = KTX2::new()
        .width(dds.get_width())
        .height(dds.get_height())
        .levels(levels)
        .vk_format(vk_format)
        .face_count(6);

    let pixel_size =
        pixel_size(vk_format).expect(&format!("no pixel_size known for {}", vk_format));

    // dds is organized like this: face x in all mips, face -x in all mips...etc.
    // ktx2 is organized like this: mip 1 with all faces, mip2 with all faces etc.
    let level_size = dds.get_mut_data(0).unwrap().len();
    let mut level_offset = 0;
    for level_index in 0..levels {
        let divisor = 1 << level_index;
        let level_width = dds.get_width() / divisor;
        let level_height = dds.get_height() / divisor;
        let face_size = level_width * level_height * pixel_size;

        let mut level_data = vec![];
        for face_index in 0..6 {
            let data = dds.get_mut_data(0).unwrap();
            let ptr_offset = level_size * face_index + level_offset;
            let data = unsafe {
                let ptr = data.as_mut_ptr();
                let ptr = ptr.add(ptr_offset);
                std::slice::from_raw_parts(ptr, face_size as usize)
            };
            level_data.extend_from_slice(data);
        }
        ktx = ktx.add_level(&level_data);
        level_offset += face_size as usize;
    }
    ktx.write(writer)?;

    Ok(())
}
