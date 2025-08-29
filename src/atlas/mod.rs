// Copyright 2025 Natalie Baker // Apache License v2 //

use std::collections::HashMap;

use bevy::{platform::hash::FixedHasher, prelude::*};

pub mod builder;

mod plugin;
pub use plugin::*;

mod slot;
pub use slot::*;

#[derive(Debug, Default, Clone, Deref, DerefMut)]
#[repr(transparent)]
pub struct TileAtlasGroup(HashMap<String, TileAtlasEntry, FixedHasher>);

#[derive(Debug, Copy, Clone)]
pub struct TileAtlasEntry {
    pub index: u16,
    pub count: u16,
}

#[derive(Debug, Clone, Asset, TypePath)]
pub struct TileAtlas {
    image:  Handle<Image>,
    lookup: HashMap<String, TileAtlasGroup, FixedHasher>,
}

impl TileAtlas {

    #[must_use]
    pub fn new(image: Handle<Image>, lookup: HashMap<String, TileAtlasGroup, FixedHasher>) -> Self {
        assert!(image.is_strong());
        Self{image, lookup}
    }

    #[must_use]
    pub const fn image(&self) -> &Handle<Image> {
        &self.image
    }

    #[must_use]
    pub fn get_group(&self, group: &str) -> Option<&TileAtlasGroup> {
        self.lookup.get(group)
    }

    #[must_use]
    pub fn get_entry(&self, group: &str, id: &str) -> Option<&TileAtlasEntry> {
        self.lookup.get(group).and_then(|g| g.get(id))
    }

}

