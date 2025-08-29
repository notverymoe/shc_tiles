// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::Read, SystemParamItem}}, render::render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}, sprite_render::SetMesh2dViewBindGroup};

use crate::sparse::render::TileGridSparseExtracted;

use super::TileGridSparseBindGroups;

pub type TileGridSparseDrawCommands = (SetItemPipeline, SetMesh2dViewBindGroup<0>, TileGridSparseDrawCommand);

pub struct TileGridSparseDrawCommand;

impl<P> RenderCommand<P> for TileGridSparseDrawCommand
where
    P: PhaseItem,
{
    type Param     = ();
    type ViewQuery = ();
    type ItemQuery = (Read<TileGridSparseExtracted>, Read<TileGridSparseBindGroups>);

    fn render<'w>(
        _: &P,
        (): ROQueryItem<'w, '_, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        (): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((bind_group, count)) = entity.and_then(|v| v.1.bind_group.as_ref().map(|b| (b, v.0.draw_count))) else { return RenderCommandResult::Skip; };
        pass.set_bind_group(1, bind_group, &[]);
        pass.draw(0..6, 0..count);
        RenderCommandResult::Success
    }
}