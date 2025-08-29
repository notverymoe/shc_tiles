// Copyright 2025 Natalie Baker // Apache License v2 //

use std::collections::HashMap;

use crate::atlas::{builder::{DownsampleBilinearSRGB, TileAtlasBuilder, TileSetSettings}, TileAtlas};

use bevy::{platform::hash::FixedHasher, prelude::*};

#[derive(Debug, Component)]
pub struct TileAtlasBuildQueueTarget {
    target: Handle<TileAtlas>,
}

impl TileAtlasBuildQueueTarget {

    #[must_use]
    pub fn new(target: Handle<TileAtlas>) -> Self {
        assert!(target.is_strong());
        Self{target}
    }

}

#[derive(Debug, Default, Component)]
pub struct TileAtlasBuildQueue {
    builder: Option<TileAtlasBuilder>,
    queue: HashMap<String, HashMap<String, TileAtlasBuildQueueImageItem, FixedHasher>, FixedHasher>,
    count_loaded: usize,
    count_total:  usize,
    queue_locked: bool,
}

#[derive(Debug, Clone)]
pub struct TileAtlasBuildQueueImageItem {
    pub handle:   Handle<Image>,
    pub settings: TileSetSettings,
}

impl TileAtlasBuildQueue {

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn new_with_size(size: u32) -> Self {
        Self { 
            builder: Some(TileAtlasBuilder::new(size)),
            ..Self::default()
        }
    }

}

impl TileAtlasBuildQueue {

    pub fn insert_image(
        &mut self,
        group_id: &str,
        tile_id:  &str,
        handle:   Handle<Image>,
        settings: TileSetSettings,
    ) {
        assert!(!self.queue_locked);
        
        let increment_count = self.queue.entry(group_id.to_owned()).or_default().insert(
            tile_id.to_owned(), 
            TileAtlasBuildQueueImageItem{
                handle,
                settings
            }
        ).is_none();

        if increment_count {
            self.count_total += 1;
        }
    }

    pub const fn lock_queue(&mut self) {
        self.queue_locked = true;
    }

    #[must_use]
    pub const fn queue_locked(&self) -> bool {
        self.queue_locked
    }

}

impl TileAtlasBuildQueue {

    #[must_use]
    pub const fn count_total(&self) -> usize {
        self.count_total
    }

    #[must_use]
    pub const fn count_loaded(&self) -> usize {
        self.count_loaded
    }

    pub fn load(&mut self, group_id: &str, tile_id: &str, src: &[u8], src_size: [u32; 2], src_settings: TileSetSettings) {
        assert!(self.queue_locked);

        let was_queued = self.queue.get_mut(group_id).and_then(|g| g.remove(tile_id)).is_some();
        if was_queued { self.count_loaded += 1; }

        let builder = self.builder.get_or_insert_with(|| {
            let tile_size = [0,1].map(|i| (src_size[i] - src_settings.offset[i])/src_settings.count[i] - src_settings.spacing[i]);
            let tile_size = tile_size[0].min(tile_size[1]);
            TileAtlasBuilder::new(tile_size)
        });

        builder.insert_tileset(group_id, tile_id, 0, src, src_size[0], src_settings);
    }

    pub fn skip(&mut self, group_id: &str, tile_id: &str) {
        let was_queued = self.queue.get_mut(group_id).and_then(|g| g.remove(tile_id)).is_some();
        if was_queued { self.count_loaded += 1; }
    }

    pub fn skip_remaining(&mut self) {
        self.count_loaded = self.count_total;
        self.queue.clear();
    }

}

impl TileAtlasBuildQueue {

    #[must_use] 
    pub const fn is_complete(&self) -> bool {
        self.queue_locked && self.count_loaded >= self.count_total
    }

    #[must_use] 
    pub fn get_size(&self) -> Option<u32> {
        self.builder.as_ref().map(|b| b.mip_level_size(0))
    }

    #[must_use]
    pub fn reset(&mut self, size: Option<u32>) -> TileAtlasBuilder {
        let builder = core::mem::replace(&mut self.builder, size.map(TileAtlasBuilder::new)).unwrap();
        self.queue_locked = false;
        self.count_loaded = 0;
        self.count_total  = 0;
        self.queue.clear();
        
        builder
    }

}

pub fn proccess_tile_atlas_build_queue(
    mut q_build_queues: Query<&mut TileAtlasBuildQueue>,
    r_images: Res<Assets<Image>>,
) {
    for mut build_queue in &mut q_build_queues {

        if !build_queue.queue_locked() { return; }

        // TODO OPT this is horrid
        let queue = build_queue.queue.clone();
        let queue = queue.iter().flat_map(|(group_id, group)| group.iter().map(move |(tile_id, item)| (group_id, tile_id, item)));

        for (group_id, tile_id, entry) in queue {
            if let Some(image) = r_images.get(&entry.handle) {
                build_queue.load(
                    group_id, 
                    tile_id, 
                    image.data.as_ref().unwrap(), 
                    [image.width(), image.height()], 
                    entry.settings
                );
            }
        }
    }

}

pub fn process_tile_atlas_build_queues_with_target(
    mut commands: Commands,
    mut q_build_queues: Query<(Entity, &TileAtlasBuildQueueTarget, &mut TileAtlasBuildQueue)>,
    mut r_images: ResMut<Assets<Image>>,
    mut r_atlas:  ResMut<Assets<TileAtlas>>,
) {
    for (entity, result, mut build_queue) in &mut q_build_queues {
        if !build_queue.is_complete() { return; }

        let (image, lookup) = {
            let mut builder = build_queue.reset(None);
            builder.downsample_levels(0..u32::MAX, false, DownsampleBilinearSRGB); // TODO HACK configurable
            (
                builder.build_image(),
                builder.build_lookup()
            )
        };

        let image = r_images.add(image);
        r_atlas.insert(&result.target, TileAtlas::new(image, lookup)).unwrap();
        commands.entity(entity).despawn();
    }
}