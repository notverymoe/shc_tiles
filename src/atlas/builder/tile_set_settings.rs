// Copyright 2025 Natalie Baker // Apache License v2 //


#[derive(Debug, Clone, Copy)]
pub struct TileSetSettings {
    pub offset:  [u32; 2],
    pub spacing: [u32; 2],
    pub count:   [u32; 2],
}

impl TileSetSettings {

    #[must_use] 
    pub const fn new(count_x: u32, count_y: u32) -> Self {
        Self { offset: [0,0], spacing: [0,0], count: [count_x, count_y] }
    }

    #[must_use] 
    pub const fn with_count(self, count_x: u32, count_y: u32) -> Self {
        Self {
            count: [count_x, count_y],
            ..self
        }
    }

    #[must_use] 
    pub const fn with_offset(self, offset_x: u32, offset_y: u32) -> Self {
        Self {
            offset: [offset_x, offset_y],
            ..self
        }
    }

    #[must_use] 
    pub const fn with_spacing(self, spacing_x: u32, spacing_y: u32) -> Self {
        Self {
            spacing: [spacing_x, spacing_y],
            ..self
        }
    }

}