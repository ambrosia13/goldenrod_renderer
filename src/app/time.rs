use std::time::{Duration, Instant};

use bevy_ecs::resource::Resource;
use bevy_ecs::system::ResMut;

#[derive(Resource, Debug)]
pub struct Time {
    last_frame: Instant,
    delta: Duration,
    frame_count: u128,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: Duration::ZERO,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        let new_instant = Instant::now();
        let delta = self.last_frame.elapsed();

        self.last_frame = new_instant;
        self.delta = delta;
        self.frame_count += 1;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    #[expect(unused)]
    pub fn frame_count(&self) -> u128 {
        self.frame_count
    }
}

pub fn update_system(mut time: ResMut<Time>) {
    time.update();
}
