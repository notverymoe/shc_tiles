// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{camera::visibility::VisibilityClass, prelude::*, render::sync_world::SyncToRenderWorld};

use crate::{prelude::TileAtlasSlot, shared::RenderPass2d, atlas::TileAtlas};

pub mod render;

mod value;
pub use value::*;

mod builder;
pub use builder::*;

#[derive(Debug, Clone, Component)]
#[require(GlobalTransform, Visibility, VisibilityClass, SyncToRenderWorld)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<TileGridSparse>)]
pub struct TileGridSparse {
    offset: Vec2,
    size:  UVec2,
    texture_atlas: Option<Handle<TileAtlas>>,
    data_change_tick: usize,
    data: Vec<TileGridSparseValue>,
    scale: f32,
    render_pass: RenderPass2d,
    y_depth_scale: f32,
}

impl TileGridSparse {

    #[must_use]
    pub const fn new(
        offset: Vec2,
        size:   UVec2,
        scale:  f32, 
        texture_atlas: Option<Handle<TileAtlas>>,
        render_pass: RenderPass2d,
        y_depth_scale: f32,
    ) -> Self {
        Self { 
            offset,
            size,
            texture_atlas,
            data_change_tick: 0, 
            data: Vec::new(), 
            scale,
            render_pass,
            y_depth_scale,
        }
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

impl TileGridSparse {

    #[must_use]
    pub const fn size(&self) -> UVec2 {
        self.size
    }

    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    #[must_use]
    pub const fn data_change_tick(&self) -> usize {
        self.data_change_tick
    }
}

impl TileGridSparse {

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
        &self.texture_atlas
    }

    pub fn set_atlas(&mut self, texture_atlas: Option<Handle<TileAtlas>>) {
        self.texture_atlas = texture_atlas;
    }

    #[must_use]
    pub const fn render_pass(&self) -> RenderPass2d {
        self.render_pass
    }

    pub const fn set_render_pass(&mut self, render_pass: RenderPass2d) {
        self.render_pass = render_pass;
    }

}

impl TileGridSparse {

    pub fn push(&mut self, pos: UVec2, value: impl Into<TileAtlasSlot>) {
        self.update_data_change_tick();
        self.data.push(TileGridSparseValue::new(pos, value));
    }

    pub fn retain<F>(&mut self, mut f: F) 
    where
        F: FnMut(UVec2, TileAtlasSlot) -> bool 
    {
        self.update_data_change_tick();
        self.data.retain(|v| f(v.position(), v.texture()));
    }

    pub fn clear(&mut self) {
        self.update_data_change_tick();
        self.data.clear();
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

}

impl TileGridSparse {

    #[must_use]
    pub const fn data(&self) -> &Vec<TileGridSparseValue> {
        &self.data
    }

    #[must_use]
    pub fn data_clone(&self) -> Box<[TileGridSparseValue]> {
        self.data.clone().into_boxed_slice()
    }

}

impl TileGridSparse {

    const fn update_data_change_tick(&mut self) {
        self.data_change_tick = self.data_change_tick.wrapping_add(1);
    }

}
