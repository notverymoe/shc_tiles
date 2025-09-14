// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::prelude::*;

use crate::atlas::TileAtlasSlot;

#[derive(Debug, Clone, Component)]
pub struct TileGridDenseData {
    data_change_tick: usize,
    data:  Box<[TileAtlasSlot]>,
    size:  UVec2,
}

impl TileGridDenseData {

    #[must_use]
    pub fn new(
        size:  UVec2,
        filled: TileAtlasSlot,
    ) -> Self {
        let len = (size.element_product() as usize).next_multiple_of(2); // Must align to an u32
        Self {
            data_change_tick: 0,
            data: vec![filled; len].try_into().unwrap(),
            size,
        }
    }

}

impl TileGridDenseData {

    pub fn fill(&mut self, pos: UVec2, size: UVec2, values: &[TileAtlasSlot]) {
        self.update_data_change_tick();
        let idx      = Self::calc_idx(self.size, pos);
        let dst_size = (size  + pos).min(self.size) - pos;
        for y in 0..dst_size.y {
            let src_start = (y*size.x) as usize;
            let dst_start = idx + (y*self.size.x) as usize;
            self.data[dst_start..dst_start+(dst_size.x as usize)].copy_from_slice(&values[src_start..src_start+(dst_size.x as usize)]);
        }
    }

    pub fn set(&mut self, pos: UVec2, value: impl Into<TileAtlasSlot>) {
        self.set_at(Self::calc_idx(self.size, pos), value);
    }

    pub fn set_at(&mut self, idx: usize, value: impl Into<TileAtlasSlot>) {
        self.update_data_change_tick();
        self.data[idx] = value.into();
    }

    #[must_use]
    pub fn get(&self, pos: UVec2) -> TileAtlasSlot {
        self.get_at(Self::calc_idx(self.size, pos))
    }

    #[must_use]
    pub fn get_at(&self, idx: usize) -> TileAtlasSlot {
        self.data[idx]
    }

}

impl TileGridDenseData {

    #[must_use]
    pub const fn size(&self) -> UVec2 {
        self.size
    }

    #[must_use]
    pub const fn data(&self) -> &[TileAtlasSlot] {
        &self.data
    }

    #[must_use]
    pub fn data_clone(&self) -> Box<[TileAtlasSlot]> {
        self.data.clone()
    }

    #[must_use]
    pub const fn data_change_tick(&self) -> usize {
        self.data_change_tick
    }

}

impl TileGridDenseData {

    const fn update_data_change_tick(&mut self) {
        self.data_change_tick = self.data_change_tick.wrapping_add(1);
    }

    #[must_use]
    const fn calc_idx(size: UVec2, pos: UVec2) -> usize {
        (pos.x + pos.y*size.x) as usize
    }

}
