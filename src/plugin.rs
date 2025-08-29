// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::app::{PluginGroup, PluginGroupBuilder};

use crate::{atlas::PluginTileAtlas, dense::render::PluginTileGridDenseRender, shared::PluginTileGridShared, sparse::render::PluginTileGridSparseRender};

#[derive(Debug)]
pub struct PluginsTileRender;

impl PluginGroup for PluginsTileRender {

    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PluginTileGridShared      )
            .add(PluginTileAtlas           )
            .add(PluginTileGridDenseRender )
            .add(PluginTileGridSparseRender)
    }

}
