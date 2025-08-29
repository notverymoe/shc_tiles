// Copyright 2025 Natalie Baker // Apache License v2 //

use std::{fs::File, io::BufReader, path::Path};

use image::{save_buffer_with_format, ExtendedColorType, ImageBuffer, ImageReader, ImageFormat, Rgba};

use shc_tiles::prelude::*;

fn main() {
    let mut img = load_image("assets/tile_wall.png");
    let size = img.width();

    let mut img_rev = bytemuck::pod_collect_to_vec::<u8, u32>(&img);
    img_rev.reverse();
    let mut img_rev = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(img.width(), img.height(), bytemuck::pod_collect_to_vec(&img_rev)).unwrap();

    // // Create Atlas // //
    let mut atlas = TileAtlasBuilder::new(size);
    for i in 0..4096 {

        let target: &mut [u8] = if i % 2 == 0 { &mut img } else { &mut img_rev };
        atlas.insert_single(
            "base", 
            &format!("image{i}"), 
            0, 
            target, 
            size, 
            [0_u32, 0_u32]
        );

        for (idx, data) in target.iter_mut().enumerate() {
            if (idx + 1) % 4 == 0 { continue; }
            *data = data.wrapping_add(idx as u8);
        }

    }
    atlas.downsample_levels(0..u32::MAX, false, DownsampleBilinearSRGB);
    // atlas.limit_levels(2);

    // // Save Atlas // //
    atlas.write_uncompressed_to(std::fs::File::create("out/atlas_uncompressed.sfa").unwrap()).unwrap();
    atlas.write_compressed_to(std::fs::File::create("out/atlas_compressed.sfa").unwrap()).unwrap();

    // // Load Atlas // //
    let atlas = TileAtlasBuilder::read_compressed_from(std::fs::File::open("out/atlas_compressed.sfa").unwrap()).unwrap();

    // // Save Images // //
    let set_a = atlas.get_tile("base", "image0").unwrap();
    let set_b = atlas.get_tile("base", "image1").unwrap();

    for level in 0..atlas.find_mip_level_common_max() {
        save_image(&format!("out/forward_mip_{level}.png"), atlas.mip_level_size(level), &set_a.levels[level as usize].data[0]);
        save_image(&format!("out/forward_mip_{level}.png"), atlas.mip_level_size(level), &set_b.levels[level as usize].data[0]);
    }

    // // Build Image // //
    let image = atlas.build_image();
    let pages = image.texture_descriptor.size.depth_or_array_layers;
    let levels = image.texture_descriptor.mip_level_count;

    for level in 0..levels {
        for page in 0..pages {
            let range = atlas.get_page_range_bytes(page, level).unwrap();
            let page_size = atlas.mip_level_size(level)*16;
            save_buffer_with_format(
                format!("out/page-dump-p{page}-l{level}.png"),
                &image.data.as_ref().unwrap()[range], 
                page_size, 
                page_size, 
                ExtendedColorType::Rgba8,
                ImageFormat::Png
            ).unwrap();
        }
    }
}

fn load_image(path: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let file = BufReader::new(File::open(path).unwrap());
    let reader = ImageReader::new(file).with_guessed_format().unwrap().decode().unwrap();
    reader.to_rgba8()
}

fn save_image(path: &str, size: u32, img: &[u8]) {
    let path = Path::new(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(size, size, Vec::from(img)).unwrap();
    img.save(path).unwrap();
}