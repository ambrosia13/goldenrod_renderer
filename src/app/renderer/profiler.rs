use bevy_ecs::resource::Resource;
use bevy_ecs::{system::Commands, world::World};
use wgputil::GpuHandle;

use crate::app::renderer::SurfaceState;
use crate::app::time::Time;

#[derive(Resource)]
pub struct RenderProfiler {
    pub time_queries: Vec<(String, wgputil::profile::TimeQuery)>,
    pub times: Vec<(String, std::time::Duration)>,
}

impl RenderProfiler {
    fn new() -> Self {
        Self {
            time_queries: Vec::new(),
            times: Vec::new(),
        }
    }

    fn read_times(&mut self, gpu: &GpuHandle) {
        // clear previously read times
        self.times.clear();

        for (name, time_query) in &self.time_queries {
            self.times
                .push((name.clone(), time_query.read(gpu).unwrap()));
        }
    }

    pub fn push(&mut self, gpu_handle: &GpuHandle, name: impl AsRef<str>) -> usize {
        let index = self.time_queries.len();
        let time_query = wgputil::profile::TimeQuery::new(&gpu_handle.device);

        self.time_queries
            .push((String::from(name.as_ref()), time_query));

        index
    }

    pub fn init(mut commands: Commands) {
        commands.insert_resource(Self::new());
    }

    pub fn post_render(world: &mut World) {
        let time = world.resource::<Time>();

        // Don't run this code every frame, for numerical stability, and so we don't
        // map buffers every frame
        if time.frame_count() % 40 != 0 {
            return;
        }

        let surface_state = world.resource::<SurfaceState>();
        let gpu = surface_state.gpu.clone();

        let mut profiler = world.resource_mut::<RenderProfiler>();
        profiler.read_times(&gpu);
    }
}
