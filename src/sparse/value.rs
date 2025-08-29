// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::math::UVec2;
use bytemuck::{Pod, Zeroable};

use crate::atlas::TileAtlasSlot;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(C, align(8))]
pub struct TileGridSparseValue {
    texture:  TileAtlasSlot,
    position: [u16; 2],
}

impl TileGridSparseValue {

    pub fn new(pos: UVec2, value: impl Into<TileAtlasSlot>) -> Self {
        let x = pos.x as u16;
        let y = pos.y as u16;
        Self {
            texture: value.into(), 
            position: [x, y]
        }
    }

    #[must_use]
    pub const fn position(self) -> UVec2 {
        UVec2::new(self.position[0] as u32, self.position[1] as u32)
    }

    #[must_use]
    pub const fn texture(self) -> TileAtlasSlot {
        self.texture
    }


}
