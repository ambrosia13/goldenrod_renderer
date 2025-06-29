use bevy_ecs::system::{Res, ResMut};
use winit::keyboard::KeyCode;

use crate::app::{camera::Camera, input::Input, menu::Menu, time::Time};

pub fn input_control(
    input: Res<Input>,
    time: Res<Time>,
    mut menu: ResMut<Menu>,
    mut camera: ResMut<Camera>,
) {
    // When the user presses escape, disable fullscreen
    if input.keys.just_pressed(KeyCode::Escape) {
        menu.settings.fullscreen = !menu.settings.fullscreen;
    }

    let mut fov_sensitivity = 2.5;
    let mut speed_sensitivity = 1.0;

    if input.keys.pressed(KeyCode::ControlLeft) {
        fov_sensitivity *= 4.0;
        speed_sensitivity *= 4.0;
    }

    if input.keys.pressed(KeyCode::ArrowUp) {
        camera.fov += fov_sensitivity * time.delta().as_secs_f32();
    }

    if input.keys.pressed(KeyCode::ArrowDown) {
        camera.fov -= fov_sensitivity * time.delta().as_secs_f32();
    }

    if input.keys.pressed(KeyCode::ArrowLeft) {
        camera.movement_speed -= speed_sensitivity * time.delta().as_secs_f32();
    }

    if input.keys.pressed(KeyCode::ArrowRight) {
        camera.movement_speed += speed_sensitivity * time.delta().as_secs_f32();
    }

    camera.fov = camera.fov.clamp(30.0, 150.0);
    camera.movement_speed = camera.movement_speed.max(0.0);
}
