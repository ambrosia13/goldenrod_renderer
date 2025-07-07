use bevy_ecs::{
    resource::Resource,
    system::{Commands, Res},
};
use glam::UVec2;
use winit::dpi::PhysicalSize;

use crate::ecs::ResourceWrapper;

pub mod final_pass;
pub mod material;
pub mod pathtrace;
pub mod profiler;

pub type SurfaceState = ResourceWrapper<wgputil::SurfaceState>;
pub type FrameRecord = ResourceWrapper<wgputil::FrameRecord>;

#[derive(Resource)]
pub struct RendererViewport {
    pub start: UVec2,
    pub end: UVec2,
}

impl RendererViewport {
    pub fn get_width(&self) -> u32 {
        self.end.x - self.start.x
    }

    pub fn get_height(&self) -> u32 {
        self.end.y - self.start.y
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.get_width(),
            height: self.get_height(),
        }
    }

    pub fn init(mut commands: Commands, surface_state: Res<SurfaceState>) {
        commands.insert_resource(Self {
            start: UVec2::new(0, 0),
            end: UVec2::new(
                surface_state.viewport_size.width,
                surface_state.viewport_size.height,
            ),
        })
    }
}
