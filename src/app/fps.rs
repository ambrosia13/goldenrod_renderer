use std::time::Duration;

use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

use super::time::Time;

pub const FPS_NUM_SAMPLES: usize = 64;

#[derive(Resource)]
pub struct FpsCounter {
    samples: [Duration; FPS_NUM_SAMPLES],
    index: usize,
    count: usize,
}

impl FpsCounter {
    fn push(&mut self, duration: Duration) {
        self.samples[self.index] = duration;
        self.index = (self.index + 1) % self.samples.len();
        self.count = (self.count + 1).min(self.samples.len());
    }

    pub fn average_fps(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let sum: Duration = self.samples.iter().take(self.count).sum();
        let average_frametime = sum / self.count as u32;

        1.0 / average_frametime.as_secs_f64()
    }

    pub fn init(world: &mut World) {
        world.insert_resource(FpsCounter::default());
    }

    pub fn update(world: &mut World) {
        let time = world.resource::<Time>();
        let delta = time.delta();

        let mut fps_counter = world.resource_mut::<FpsCounter>();
        fps_counter.push(delta);
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            samples: [Default::default(); FPS_NUM_SAMPLES],
            index: Default::default(),
            count: Default::default(),
        }
    }
}
