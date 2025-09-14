// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{camera::visibility::VisibilityClass, prelude::*, render::sync_world::SyncToRenderWorld};

use crate::{atlas::{TileAtlas, TileAtlasSlot}, shared::RenderPass2d};

pub mod render;

mod builder;
pub use builder::*;

#[derive(Debug, Clone, Component)]
#[require(GlobalTransform, Visibility, VisibilityClass, SyncToRenderWorld)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<TileGridDense>)]
pub struct TileGridDense {
    offset: Vec2,
    atlas: Option<Handle<TileAtlas>>,
    data_change_tick: usize,
    data:  Box<[TileAtlasSlot]>,
    size:  UVec2,
    scale: f32,
    render_pass: RenderPass2d,
    y_depth_scale: f32,
}

impl TileGridDense {

    #[must_use]
    pub fn new(
        offset: Vec2,
        size:  UVec2,
        scale: f32,
        atlas: Option<Handle<TileAtlas>>,
        filled: TileAtlasSlot,
        render_pass: RenderPass2d,
        y_depth_scale: f32,
    ) -> Self {
        let len = (size.element_product() as usize).next_multiple_of(2); // Must align to an u32
        Self {
            offset,
            atlas,
            data_change_tick: 0,
            data: vec![filled; len].try_into().unwrap(),
            size,
            scale,
            render_pass,
            y_depth_scale,
        }
    }


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

    #[must_use]
    const fn calc_idx(size: UVec2, pos: UVec2) -> usize {
        (pos.x + pos.y*size.x) as usize
    }

    #[must_use]
    pub fn calculate_bounds(&self) -> [Vec3; 2] {
        let p0 = self.scale*self.offset;
        let p1 = p0 + self.scale*self.size.as_vec2();

        let p0 = p0.extend(0.0);
        let p1 = p1.extend(self.y_depth_scale*(self.size.y as f32));

        [p0.min(p1), p0.max(p1)]
    }

}

impl TileGridDense {

    #[must_use]
    pub const fn size(&self) -> UVec2 {
        self.size
    }

    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
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

impl TileGridDense {

    #[must_use]
    pub const fn offset(&self) -> Vec2 {
        self.offset
    }

    pub const fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    #[must_use]
    pub const fn y_depth_scale(&self) -> f32 {
        self.y_depth_scale
    }

    pub const fn set_y_depth_scale(&mut self, y_depth_scale: f32) {
        self.y_depth_scale = y_depth_scale;
    }

    #[must_use]
    pub const fn atlas(&self) -> &Option<Handle<TileAtlas>> {
        &self.atlas
    }

    pub fn set_atlas(&mut self, atlas: Option<Handle<TileAtlas>>) {
        self.atlas = atlas;
    }

    #[must_use]
    pub const fn render_pass(&self) -> RenderPass2d {
        self.render_pass
    }

    pub const fn set_render_pass(&mut self, render_pass: RenderPass2d) {
        self.render_pass = render_pass;
    }

}

impl TileGridDense {

    #[must_use]
    pub fn local_from_world(&self, position: Vec2) -> Option<UVec2> {
        let position = position*self.scale;
        (
            position.x >= 0.0 && 
            position.x < (self.size.x as f32) &&  
            position.y >= 0.0 && 
            position.y < (self.size.y as f32)
        ).then(|| position.as_uvec2())
    }

    #[must_use]
    pub fn world_from_local(&self, position: UVec2) -> Vec2 {
        position.as_vec2()/self.scale
    }

}

impl TileGridDense {

    const fn update_data_change_tick(&mut self) {
        self.data_change_tick = self.data_change_tick.wrapping_add(1);
    }

}