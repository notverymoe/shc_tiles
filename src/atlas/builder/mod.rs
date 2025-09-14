// Copyright 2025 Natalie Baker // Apache License v2 //

use core::ops::Range;
use std::{collections::{hash_map::Entry, HashMap}, io::{Read, Write}};

use bitcode::{Decode, Encode};

use bevy::{asset::RenderAssetUsages, image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor}, platform::hash::FixedHasher, render::render_resource::{Extent3d, TextureAspect, TextureDataOrder, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension}};

mod tile_set_settings;
pub use tile_set_settings::*;

mod queue;
pub use queue::*;

mod downsample;
pub use downsample::*;

use crate::atlas::{TileAtlasEntry, TileAtlasGroup};

#[derive(Debug)]
pub enum TileAtlasBuilderReadError {
    Io(std::io::Error),
    Decode(bitcode::Error),
}

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct TileAtlasBuilderGroup {
    pub tile_sets: HashMap<String, TileAtlasBuilderSet, FixedHasher>
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TileAtlasBuilderSet {
    pub levels: Box<[TileAtlasBuilderImageSequence]>,
}

impl TileAtlasBuilderSet {

    pub fn new(level_count: u32) -> Self {
        let mut levels = Vec::with_capacity(level_count as usize);
        levels.resize_with(level_count as usize, TileAtlasBuilderImageSequence::default);
        Self{ levels: levels.into_boxed_slice() }
    }

}

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct TileAtlasBuilderImageSequence {
    pub data: Vec<Box<[u8]>>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TileAtlasBuilder {
    size:   u32,
    groups: HashMap<String, TileAtlasBuilderGroup, FixedHasher>,
}

impl TileAtlasBuilder {

    /**
     * Creates a new tile atlas build with the given mip level 0 size.
     */
    #[must_use]
    pub fn new(size: u32) -> Self {
        Self{
            size,
            groups: HashMap::default()
        }
    }

}

impl TileAtlasBuilder {

    /**
     * Gets the group with the given `group_id`
     */
    #[must_use]
    pub fn get_group(&self, group_id: &str) -> Option<&TileAtlasBuilderGroup> {
        self.groups.get(group_id)
    }

    /**
     * Gets the tile with the given `tile_id` in the group with the given `group_id`
     */
    #[must_use]
    pub fn get_tile(&self, group_id: &str, tile_id: &str) -> Option<&TileAtlasBuilderSet> {
        self.groups.get(group_id).and_then(|v| v.tile_sets.get(tile_id))
    }

}

impl TileAtlasBuilder {

    /**
     * Inserts a single image `src` at the specified mip `level` for the given `group_id` and `tile_id`.
     * - Clears the sequence of images at the specified mip level but does not affect any other level of the tile.
     * - Image data is expected to be in RGBA8 format and in row-major order.
     * - `src_width` is the width of a row in the `src` image data
     * - A slice of pixels the size of the selected mip level will be extracted from the given `src_offset` into the `src` image
     */
    pub fn insert_single(
        &mut self, 
        group_id: &str, 
        tile_id: &str,
        level: u32,
        src: &[u8],
        src_width: u32,
        src_offset: [u32; 2]
    ) {
        self.insert(
            group_id,
            tile_id,
            level,
            core::iter::once((src, src_width, src_offset))
        );
    }

    /**
     * Inserts multiple X by Y slices from a single image `src` at the specified mip `level` for the given `group_id` and `tile_id`.
     * - Clears the sequence of images at the specified mip level but does not affect any other level of the tile.
     * - Image data is expected to be in RGBA8 format and in row-major order.
     * - `src_width` is the width of a row in the `src` image data.
     * - See `src_settings` for information regarding how slicing is performed.
     */
    pub fn insert_tileset(
        &mut self, 

        group_id: &str, 
        tile_id:    &str,
        level: u32,

        src: &[u8],
        src_width: u32,
        src_settings: TileSetSettings,
    ) {
        let level_size = self.mip_level_size(level);
        let tile_count = src_settings.count[0]*src_settings.count[1];
        self.insert(
            group_id,
            tile_id,
            level,
            (0..tile_count).map(|i| {
                let x = i % src_settings.count[0];
                let y = i / src_settings.count[0];
                (
                    src,
                    src_width,
                    [
                        src_settings.offset[0] + x*src_settings.spacing[0] + x*level_size,
                        src_settings.offset[1] + y*src_settings.spacing[1] + y*level_size,
                    ]
                )
            })
        );
    }

}

impl TileAtlasBuilder {

    /**
     * Inserts a sequence of `images` at the specified mip `level` for the given `group_id` and `tile_id`.
     * - Clears the sequence of images at the specified mip level but does not affect any other level of the tile.
     * - Images provided using an iterator, see `Self::insert_single` and `Self::insert_tileset` for helpers.
     * - Iterator provides (`image_data`, `image_row_width_pixels`, [`offset_x`, `offset_y`]).
     *     - A section of pixels the size of the specified level will be cut out from the given xy-offset into the image data.
     *     - Data is expected to be in RGBA8 format and in row-major order.
     */
    pub fn insert<'a>(
        &mut self, 

        group_id: &str, 
        tile_id:  &str,
        level:    u32,

        images: impl Iterator<Item = (&'a [u8], u32, [u32; 2])>
    ) {
        let level_count = self.mip_levels_max();
        assert!(level_count >= level);

        let dst_width   = self.mip_level_size(level);
        let dst_len     = self.mip_level_data_len(level) as usize;

        let group = self.groups.entry(group_id.to_owned()).or_default();
        let set   = group.tile_sets.entry(tile_id.to_owned()).or_insert_with(|| TileAtlasBuilderSet::new(level_count));
        let sequence = &mut set.levels[level as usize];
        sequence.data.clear();

        for (src, src_width, [src_x, src_y]) in images {
            let data = if src_x == 0 && src_width == dst_width {
                // Fast copy, widths match
                let y_off = (src_y*src_width*4) as usize;
                Box::from(&src[y_off..y_off+dst_len])
            } else {
                // Row-by-row copy
                let mut dst = vec![0_u8; dst_len].into_boxed_slice();
                for dst_y in 0..dst_width {
                    rgba8_image_copy_row(
                        dst_y,
                             src, src_width, src_x, src_y, 
                        &mut dst, dst_width,     0, dst_y,
                    );
                }
                dst
            };
            sequence.data.push(data);
        }
    }

}

impl TileAtlasBuilder {
    
    /**
     * Merges another atlas builder into this one, overwriting all shared entries.
     */
    pub fn merge(&mut self, other: TileAtlasBuilder) {
        for (group_id, group) in other.groups {
            match self.groups.entry(group_id) {
                Entry::Occupied(mut o) => {
                    let dst_group = o.get_mut();
                    dst_group.tile_sets.extend(group.tile_sets);
                },
                Entry::Vacant(v) => {
                    v.insert(group);
                }
            }

        }
    }

}

impl TileAtlasBuilder {

    /**
     * Removes the tile with given `tile_id` and `group_id`, returning the removed value.
     * The group will also be removed if it has no tiles.
     */
    pub fn remove(&mut self, group_id: &str, tile_id: &str) -> Option<TileAtlasBuilderSet> {
        if let Some(group) = self.groups.get_mut(group_id) {
            let result = group.tile_sets.remove(tile_id);
            if group.tile_sets.is_empty() { self.groups.remove(group_id); }
            result
        } else {
            None
        }
    }

    /**
     * Removes a single mip `level` from a tile with given `tile_id` and `group_id`, returning the removed value.
     * The tile will also be removed if it has no data on any level.
     */
    pub fn remove_level(&mut self, group_id: &str, tile_id: &str, level: u32) -> Option<TileAtlasBuilderImageSequence> {
        if level >= self.mip_levels_max() { return None; }
        if let Some(set) = self.groups.get_mut(group_id).and_then(|g| g.tile_sets.get_mut(tile_id)) {
            let result = Some(core::mem::take(&mut set.levels[level as usize]));
            if set.levels.iter().all(|s| s.data.is_empty()) { self.remove(group_id, tile_id); }
            result
        } else {
            None
        }
    }

    /**
     * Removes a range of mip `levels` from a tile with given `tile_id` and `group_id`.
     * The tile will also be removed if it has no data on any level.
     */
    pub fn remove_levels(&mut self, group_id: &str, tile_id: &str, range: Range<u32>) {
        let Some(set) = self.groups.get_mut(group_id).and_then(|g| g.tile_sets.get_mut(tile_id)) else { return; };
        for level in range {
            if level as usize >= set.levels.len() { break; }
            set.levels[level as usize].data.clear();
        }
    }

}

impl TileAtlasBuilder {

    pub fn limit_levels(&mut self, max_level: u32) {
        for group in self.groups.values_mut() {
            for set in group.tile_sets.values_mut() {
                set.levels[(max_level as usize)..].fill_with(TileAtlasBuilderImageSequence::default);
            }
        }
    }

    pub fn downsample_levels(
        &mut self,
        levels: Range<u32>,
        force: bool,
        downsampler: impl DownsampleAlgorithm,
    ) {
        let mip_levels_max = self.mip_levels_max();
        for group in self.groups.values_mut() {
            for set in group.tile_sets.values_mut() {
                Self::downsample_levels_in_set(
                    self.size,
                    mip_levels_max,
                    set,
                    levels.clone(),
                    force,
                    &downsampler
                );
            }
        }
    }
    /**
     * Creates mip `levels` for the given range in the tile with the
     * given `tile_id` and `group_id`, using the provided `downsampler`
     * implementation.
     */
    pub fn downsample_levels_for(
        &mut self,

        group_id: &str, 
        tile_id:  &str,
        levels:   Range<u32>,
        force: bool,

        downsampler: impl DownsampleAlgorithm,
    ) {
        Self::downsample_levels_in_set(
            self.size,
            self.mip_levels_max(),
            self.groups.get_mut(group_id).and_then(|g| g.tile_sets.get_mut(tile_id)).unwrap(),
            levels,
            force,
            &downsampler
        );
    }

    fn downsample_levels_in_set(
        base_size: u32,
        mip_levels_max: u32,

        set: &mut TileAtlasBuilderSet,
        levels: Range<u32>,
        force: bool,

        downsampler: &impl DownsampleAlgorithm,
    ) {
        for level in levels.skip(1) {
            if level >= mip_levels_max { return; }

            let [prev, current] = set.levels.get_disjoint_mut([(level-1) as usize, level as usize]).unwrap();
            let src_size = mip_level_size(base_size, level-1) as usize;
            let dst_len  = mip_level_data_len(base_size, level) as usize;

            if force {
                current.data.clear();
            }

            for src in prev.data.iter().skip(current.data.len()) {
                let mut dst = vec![0_u8; dst_len].into_boxed_slice();
                downsampler.downsample(src, src_size, &mut dst);
                current.data.push(dst);
            }
        }
    }
    
}

impl TileAtlasBuilder {

    #[must_use]
    pub fn build_lookup(&self) -> HashMap<String, TileAtlasGroup, FixedHasher> {
        let mut index = 0;
        let mut lookup = HashMap::<String, TileAtlasGroup, FixedHasher>::default();
        for (group_name, group_src) in &self.groups {
            let mut group_dst = TileAtlasGroup::default();
            for (set_name, set) in &group_src.tile_sets {
                if set.levels[0].data.is_empty() { continue; }

                group_dst.insert(set_name.clone(), TileAtlasEntry{
                    index,
                    count: set.levels[0].data.len() as u16
                });
                index += set.levels[0].data.len() as u16;
            }

            if group_dst.is_empty() { continue; }
            lookup.insert(group_name.clone(), group_dst);
        }
        lookup
    }

    #[must_use]
    pub fn build_image(&self) -> Image {
        self.build_image_with_settings(
            Some("tile_atlas_texture"),
            TextureUsages::TEXTURE_BINDING,
            RenderAssetUsages::RENDER_WORLD,
            ImageSampler::Descriptor(ImageSamplerDescriptor{
                label: Some("tile_atlas_texture_sampler".to_owned()),
                address_mode_u: ImageAddressMode::ClampToEdge,
                address_mode_v: ImageAddressMode::ClampToEdge,
                address_mode_w: ImageAddressMode::ClampToEdge,
                mag_filter:    ImageFilterMode::Nearest,
                min_filter:    ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 32.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            })
        )
    }

    #[must_use]
    pub fn build_image_with_settings(
        &self,
        texture_label: Option<&'static str>,
        texture_usage: TextureUsages,
        asset_usage:   RenderAssetUsages,
        sampler:       ImageSampler
    ) -> Image {
        let mip_level_count = self.find_mip_level_common_max();

        let page_count = self.page_count();
        let total_len  = (page_count as usize) * (0..mip_level_count).map(|i| (self.mip_level_data_len(i)*16*16) as usize).sum::<usize>();
        let mut dst    = Vec::<u8>::with_capacity(total_len);

        for page in 0..page_count {
            for level in 0..mip_level_count {
                self.build_page(&mut dst, level, page);
            }
        }

        Image{
            data: Some(dst),
            data_order: TextureDataOrder::LayerMajor,
            texture_descriptor: TextureDescriptor { 
                label: texture_label, 
                size: Extent3d {
                    width:  self.size * 16,
                    height: self.size * 16,
                    depth_or_array_layers: page_count,
                }, 
                mip_level_count, 
                sample_count: 1, 
                dimension: TextureDimension::D2, 
                format: TextureFormat::Rgba8UnormSrgb, 
                usage: texture_usage, 
                view_formats: &[],
            },
            texture_view_descriptor: Some(TextureViewDescriptor { 
                label: None, 
                format: None, 
                dimension: Some(TextureViewDimension::D2Array), 
                usage: None, 
                aspect: TextureAspect::All, 
                base_mip_level: 0, 
                mip_level_count: None, 
                base_array_layer: 0, 
                array_layer_count: None
            }),
            asset_usage,
            sampler,
            copy_on_resize: false,
        }

    }

    #[must_use]
    pub fn mip_levels_complete(&self) -> bool {
        self.groups.values().all(|g| g.tile_sets.values().all(|s| {
            let ([base], mips) = s.levels.split_at(1) else { return false; };
            mips.iter().all(|m| m.data.len() == base.data.len())
        }))
    }

    #[must_use]
    pub fn image_count(&self) -> u32 {
        self.groups.values().flat_map(|g| g.tile_sets.values().map(|s| s.levels[0].data.len())).sum::<usize>() as u32
    }

    #[must_use]
    pub fn page_count(&self) -> u32 {
        self.image_count().div_ceil(16*16)
    }

    #[must_use]
    pub const fn page_len(&self, level: u32) -> u32 {
        self.mip_level_data_len(level)*16*16
    }

    pub fn build_page(&self, dst: &mut Vec<u8>, level: u32, page: u32) {
        // Allocate Page //
        // TODO OPT version where these values are provided.
        let page_start = dst.len();
        let page_len   = self.page_len(level) as usize;
        dst.resize(dst.len() + page_len, 0);
        let dst_page = &mut dst[page_start..page_start+page_len];

        // Copy Images //
        let images     = self.groups.values().flat_map(|g| g.tile_sets.values().flat_map(|s| s.levels[level as usize].data.iter())).skip((page*16*16) as usize).take(16*16);
        let level_size = self.mip_level_size(level);
        for (idx, src) in images.enumerate() {
            let x = (idx &  0x0F) as u32;
            let y = (idx >>    4) as u32;

            let dst_x = x*level_size;
            let dst_y = y*level_size;

            for row in 0..level_size {
                rgba8_image_copy_row(
                    level_size, 
                    src,         level_size,     0, row, 
                    dst_page, 16*level_size, dst_x, row+dst_y
                );
            }
        }
    }
    
}

impl TileAtlasBuilder {

    /**
     * Calculates the maximium number of mip levels including the base level
     */
    #[must_use]
    pub const fn mip_levels_max(&self) -> u32 {
        mip_levels_max(self.size)
    }

    /**
     * Calculates the lowest supported mip map level of all loaded tiles.
     * It is safe to generate mipmaps up to this level.
     */
    #[must_use]
    pub fn find_mip_level_common_max(&self) -> u32 {
        self.groups.values().flat_map(|g| g.tile_sets.values().map(|s| {
            let ([base], levels) = s.levels.split_at(1) else { return 0; };
            levels.iter().enumerate().find(|l| base.data.len() != l.1.data.len()).map_or(levels.len(), |v| v.0) + 1
        })).min().unwrap_or(0) as u32
    }

    /**
     * Calculates the size of a tile in the given mip level
     */
    #[must_use]
    pub const fn mip_level_size(&self, level: u32) -> u32 {
        mip_level_size(self.size, level)
    }

    /**
     * Calculates the size of a tile's image in the given mip level
     */
    #[must_use]
    pub const fn mip_level_data_len(&self, level: u32) -> u32 {
        mip_level_data_len(self.size, level)
    }

    /**
     * Returns the byte range in the resulting built image for a particular page and mip level.
     */
    #[must_use]
    pub fn get_page_range_bytes(&self, page: u32, level: u32) -> Option<Range<usize>> {
        let mip_levels_max = self.find_mip_level_common_max();
        if level >= mip_levels_max { return None; }
        if page >= self.page_count() { return None; }

        let page  = page as usize;
        let level = level as usize;
        
        let size_mips = (0..mip_levels_max).map(|i| (self.mip_level_data_len(i)*16*16) as usize).collect::<Vec<_>>();

        let page_offset_base  = size_mips.iter().sum::<usize>()*page;
        let page_offset_inner = size_mips.iter().take(level).sum::<usize>();

        let page_start = page_offset_base + page_offset_inner;
        Some(page_start..page_start+size_mips[level])
    }
    
}

const COMPRESSION_BUFFER_SIZE: usize = 8*1024*1024; // 8 MiB

impl TileAtlasBuilder {

    pub fn write_compressed_to(&self, sink: impl Write) -> Result<(), std::io::Error> {
        brotli::CompressorWriter::new(sink, COMPRESSION_BUFFER_SIZE, 7, 24)
            .write_all(&bitcode::encode(self))
    }

    pub fn read_compressed_from(source: impl Read) -> Result<Self, TileAtlasBuilderReadError> {
        let mut vec = Vec::with_capacity(COMPRESSION_BUFFER_SIZE);
        brotli::Decompressor::new(source, COMPRESSION_BUFFER_SIZE).read_to_end(&mut vec).map_err(TileAtlasBuilderReadError::Io)?;
        bitcode::decode(&vec).map_err(TileAtlasBuilderReadError::Decode)
    }

    pub fn write_uncompressed_to(&self, mut sink: impl Write) -> Result<(), std::io::Error> {
        sink.write_all(&bitcode::encode(self))
    }

    pub fn read_uncompressed_from(mut source: impl Read) -> Result<Self, TileAtlasBuilderReadError> {
        let mut vec = Vec::with_capacity(COMPRESSION_BUFFER_SIZE);
        source.read_to_end(&mut vec).map_err(TileAtlasBuilderReadError::Io)?;
        bitcode::decode(&vec).map_err(TileAtlasBuilderReadError::Decode)
    }

}

/**
 * Calculates the maximium number of mip levels including the base level
 */
#[must_use]
const fn mip_levels_max(size: u32) -> u32 {
    size.trailing_zeros() + 1
}

/**
 * Calculates the size of a tile in the given mip level
 */
#[must_use]
const fn mip_level_size(size: u32, level: u32) -> u32 {
    size.unbounded_shr(level)
}

/**
 * Calculates the size of a tile's image in the given mip level
 */
#[must_use]
const fn mip_level_data_len(size: u32, level: u32) -> u32 {
    mip_level_size(size, level).pow(2) * 4
}

/**
 * Copies a row of the destination width from the source image at the given position into the destination image in the specified row.
 * Assumes each pixel is 4-bytes long.
 */
fn rgba8_image_copy_row(
    copy_width: u32,

    src: &[u8],
    src_width: u32,
    src_x: u32,
    src_y: u32,

    dst: &mut [u8],
    dst_width: u32,
    dst_x: u32,
    dst_y: u32,
) {
    assert!(src_x + copy_width <= src_width,      "Attempt to copy from image out of bounds (x-axis)");
    assert!(src_y * src_width < src.len() as u32, "Attempt to copy from image out of bounds (y-axis)");

    assert!(dst_x + copy_width <= dst_width,       "Attempt to copy to image out of bounds (x-axis)");
    assert!(dst_y * dst_width  < dst.len() as u32, "Attempt to copy to image out of bounds (y-axis)");

    let src_start = 4*(src_y*src_width + src_x) as usize;
    let dst_start = 4*(dst_y*dst_width + dst_x) as usize;

    let src_end = src_start + (copy_width as usize)*4;
    let dst_end = dst_start + (copy_width as usize)*4;

    dst[dst_start..dst_end].copy_from_slice(&src[src_start..src_end]);
}
