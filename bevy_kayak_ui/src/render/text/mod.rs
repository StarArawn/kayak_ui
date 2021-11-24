use bevy::{
    prelude::{
        AddAsset, AssetEvent, Assets, Commands, EventReader, Handle, HandleUntyped, Plugin, Res,
        ResMut,
    },
    reflect::TypeUuid,
    render2::{
        render_phase::DrawFunctions,
        render_resource::Shader,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
    utils::HashSet,
};

use crate::render::{
    text::{
        font_mapping::FontMapping,
        pipeline::{DrawText, TextMeta, TextPipeline},
    },
    ui_pass::TransparentUI,
};

use self::{font::KayakFont, font_texture_cache::FontTextureCache};

mod font;
mod font_mapping;
mod font_texture_cache;
mod pipeline;

pub const TEXT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1561765330850392701);

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let text_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(TEXT_SHADER_HANDLE, text_shader);

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<TextPipeline>()
            .init_resource::<TextMeta>()
            .add_system_to_stage(RenderStage::Extract, pipeline::extract_texts)
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_texts)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_texts);

        let draw_text = DrawText::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_text);

        render_app
            .init_resource::<FontTextureCache>()
            .init_resource::<ExtractedFonts>()
            .add_system_to_stage(RenderStage::Extract, extract_fonts)
            .add_system_to_stage(RenderStage::Prepare, prepare_fonts)
            .add_system_to_stage(RenderStage::Queue, create_and_update_font_cache_texture);

        app.add_asset::<KayakFont>()
            .init_resource::<FontMapping>()
            .add_startup_system(load_fonts);
    }
}

#[derive(Default)]
pub struct ExtractedFonts {
    pub fonts: Vec<(Handle<KayakFont>, KayakFont)>,
}

fn load_fonts(mut font_assets: ResMut<Assets<KayakFont>>, mut font_mapping: ResMut<FontMapping>) {
    let font_bytes = include_bytes!("../../../../resources/Roboto-Regular.ttf");
    let font = kayak_font::Font::new(font_bytes, 128);

    let handle = font_assets.add(KayakFont { font });
    font_mapping.add(handle);

    dbg!("Loaded base font!");
}

fn extract_fonts(
    mut commands: Commands,
    font_assets: Res<Assets<KayakFont>>,
    mut events: EventReader<AssetEvent<KayakFont>>,
) {
    let mut extracted_fonts = ExtractedFonts { fonts: Vec::new() };
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                changed_assets.insert(handle);
                dbg!("New font added!");
            }
            AssetEvent::Modified { handle } => {
                changed_assets.insert(handle);
                dbg!("Font changed!");
            }
            AssetEvent::Removed { handle } => {
                if !changed_assets.remove(handle) {
                    removed.push(handle.clone_weak());
                }
            }
        }
    }

    for handle in changed_assets {
        let font_asset = font_assets.get(handle).unwrap();
        let font = font_asset.clone();

        extracted_fonts.fonts.push((handle.clone_weak(), font));
    }

    commands.insert_resource(extracted_fonts);
}

fn prepare_fonts(
    mut extracted_fonts: ResMut<ExtractedFonts>,
    mut font_texture_cache: ResMut<FontTextureCache>,
) {
    for (handle, font) in extracted_fonts.fonts.drain(..) {
        font_texture_cache.add(handle, font);
    }
}

fn create_and_update_font_cache_texture(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: Res<TextPipeline>,
    mut font_texture_cache: ResMut<FontTextureCache>,
) {
    font_texture_cache.process_new(&device, &pipeline);
    font_texture_cache.process_updated(&queue);
}
