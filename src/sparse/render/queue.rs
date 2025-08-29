// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{asset::AssetId, core_pipeline::core_2d::{AlphaMask2d, AlphaMask2dBinKey, BatchSetKey2d, Opaque2d, Opaque2dBinKey, Transparent2d}, ecs::{component::Tick, query::Without, system::{Local, Query, Res, ResMut}}, math::FloatOrd, mesh::Mesh, render::{render_phase::{BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItemExtraIndex, ViewBinnedRenderPhases, ViewSortedRenderPhases}, render_resource::{PipelineCache, SpecializedRenderPipelines}, view::{ExtractedView, Msaa, RenderVisibleEntities}}, sprite_render::Mesh2dPipelineKey};

use crate::{shared::RenderPass2d, sparse::{render::{PreparedTileGridSparse, TileGridSparseDrawCommands, TileGridSparsePipeline, TileGridSparsePipelineKey}, TileGridSparse}};

pub fn tile_grid_sparse_queue_draw_commands(
    mut pipelines: ResMut<SpecializedRenderPipelines<TileGridSparsePipeline>>,
    pipeline_cache: Res<PipelineCache>,

    mut render_phases_opaque:      ResMut<ViewBinnedRenderPhases<Opaque2d>>,
    mut render_phases_mask:        ResMut<ViewBinnedRenderPhases<AlphaMask2d>>,
    mut render_phases_transparent: ResMut<ViewSortedRenderPhases<Transparent2d>>,

    draw_functions_opaque:      Res<DrawFunctions<Opaque2d>>,
    draw_functions_mask:        Res<DrawFunctions<AlphaMask2d>>,
    draw_functions_transparent: Res<DrawFunctions<Transparent2d>>,

    r_tile_grid_pipeline: Option<Res<TileGridSparsePipeline>>,
    q_tile_grid: Query<&PreparedTileGridSparse, Without<RenderVisibleEntities>>,

    mut views: Query<
        (&RenderVisibleEntities, &ExtractedView, &Msaa), 
        Without<PreparedTileGridSparse>
    >,

    mut next_tick: Local<Tick>,
) {
    let Some(tile_grid_sparse_pipeline) = r_tile_grid_pipeline else { return; };
    let draw_tile_grid_sparse_transparent = draw_functions_transparent.read().id::<TileGridSparseDrawCommands>();
    let draw_tile_grid_sparse_mask        = draw_functions_mask.read().id::<TileGridSparseDrawCommands>();
    let draw_tile_grid_sparse_opaque      = draw_functions_opaque.read().id::<TileGridSparseDrawCommands>();

    for (visible_entities, view, msaa) in &mut views {
        let Some(phase_transparent) = render_phases_transparent.get_mut(&view.retained_view_entity) else { continue; };
        let Some(phase_mask       ) =        render_phases_mask.get_mut(&view.retained_view_entity) else { continue; };
        let Some(phase_opaque     ) =      render_phases_opaque.get_mut(&view.retained_view_entity) else { continue; };

        let mesh_key = Mesh2dPipelineKey::from_hdr(view.hdr) | Mesh2dPipelineKey::from_msaa_samples(msaa.samples());

        for &(render_entity, main_entity) in visible_entities.get::<TileGridSparse>() {
            let Ok(grid) = q_tile_grid.get(render_entity) else { continue; };

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &tile_grid_sparse_pipeline,
                TileGridSparsePipelineKey{
                    mesh_key,
                    blend: grid.render_pass.into(),
                },
            );

            match grid.render_pass {
                RenderPass2d::Opaque => {
                    let this_tick = next_tick.get() + 1;
                    next_tick.set(this_tick);

                    phase_opaque.add(
                        BatchSetKey2d{indexed: false},
                        Opaque2dBinKey{
                            pipeline: pipeline_id,
                            draw_function: draw_tile_grid_sparse_opaque,
                            asset_id: AssetId::<Mesh>::invalid().untyped(),
                            material_bind_group_id: None,
                        },
                        (render_entity, main_entity),
                        InputUniformIndex::default(),
                        BinnedRenderPhaseType::NonMesh,
                        *next_tick,
                    );
                },
                RenderPass2d::AlphaMask => {
                    let this_tick = next_tick.get() + 1;
                    next_tick.set(this_tick);

                    phase_mask.add(
                        BatchSetKey2d{indexed: false},
                        AlphaMask2dBinKey{
                            pipeline: pipeline_id,
                            draw_function: draw_tile_grid_sparse_mask,
                            asset_id: AssetId::<Mesh>::invalid().untyped(),
                            material_bind_group_id: None,
                        },
                        (render_entity, main_entity),
                        InputUniformIndex::default(),
                        BinnedRenderPhaseType::NonMesh,
                        *next_tick,
                    );
                },
                RenderPass2d::Transparent => {
                    let sort_key = FloatOrd(grid.depth);
                    phase_transparent.add(Transparent2d {
                        draw_function: draw_tile_grid_sparse_transparent,
                        pipeline: pipeline_id,
                        entity: (render_entity, main_entity),
                        sort_key,
                        batch_range: 0..1,
                        extra_index: PhaseItemExtraIndex::None,
                        extracted_index: usize::MAX,
                        indexed: false,
                    });
                },
            }
        }
    }
}
