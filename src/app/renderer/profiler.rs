use bevy_ecs::{
    system::{Commands, Resource},
    world::World,
};

use crate::render::{timestamp::TimeQuery, GpuHandle};

#[derive(Resource)]
pub struct RenderProfiler {
    pub time_queries: Vec<(String, TimeQuery)>,
    pub times: Vec<(String, std::time::Duration)>,
}

impl RenderProfiler {
    fn new() -> Self {
        Self {
            time_queries: Vec::new(),
            times: Vec::new(),
        }
    }

    fn read_times(&mut self) {
        // clear previously read times
        self.times.clear();

        for (name, time_query) in &self.time_queries {
            self.times.push((name.clone(), time_query.read()));
        }
    }

    pub fn push(&mut self, gpu_handle: &GpuHandle, name: impl AsRef<str>) -> usize {
        let index = self.time_queries.len();
        let time_query = TimeQuery::new(gpu_handle);

        self.time_queries
            .push((String::from(name.as_ref()), time_query));

        index
    }

    pub fn init(mut commands: Commands) {
        commands.insert_resource(Self::new());
    }

    pub fn post_render(world: &mut World) {
        let mut profiler = world.resource_mut::<RenderProfiler>();
        profiler.read_times();
    }
}
