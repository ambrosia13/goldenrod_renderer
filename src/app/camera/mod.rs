use bevy_ecs::resource::Resource;
use bevy_ecs::{
    event::EventReader,
    system::{Commands, Res, ResMut},
};
use glam::{DVec2, Mat3, Mat4, Quat, Vec3};
use winit::{dpi::PhysicalSize, keyboard::KeyCode};

use crate::app::events::MouseMotion;
use crate::app::renderer::RendererViewport;

use super::{input::Input, time::Time};

pub mod binding;

#[derive(Resource)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,

    pub fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    pitch: f64,
    yaw: f64,

    pub movement_speed: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        look_at: Vec3,

        fov: f32,
        window_size: PhysicalSize<u32>,

        near: f32,
        far: f32,

        movement_speed: f32,
    ) -> Self {
        let (rotation, yaw, pitch) = Self::get_rotation_from_view_vector(position, look_at);

        Self {
            position,
            rotation,
            fov,
            aspect: window_size.width as f32 / window_size.height as f32,
            near,
            far,
            pitch,
            yaw,
            movement_speed,
        }
    }

    pub fn reconfigure_aspect(&mut self, window_size: PhysicalSize<u32>) {
        self.aspect = window_size.width as f32 / window_size.height as f32;
    }

    #[expect(unused)]
    pub fn look_at(&mut self, target: Vec3) {
        let (rotation, yaw, pitch) = Self::get_rotation_from_view_vector(self.position, target);

        self.rotation = rotation;
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn forward_xz(&self) -> Vec3 {
        let forward = self.forward();
        Vec3::new(forward.x, 0.0, forward.z).normalize()
    }

    pub fn right(&self) -> Vec3 {
        -(self.rotation * Vec3::X)
    }

    pub fn right_xz(&self) -> Vec3 {
        let right = self.right();
        Vec3::new(right.x, 0.0, right.z).normalize()
    }

    pub fn up(&self) -> Vec3 {
        -(self.rotation * Vec3::Y)
    }

    fn yaw_quat(&self) -> Quat {
        Quat::from_rotation_y(self.yaw.to_radians() as f32)
    }

    fn pitch_quat(&self) -> Quat {
        Quat::from_rotation_x(self.pitch.to_radians() as f32)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward(), Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    pub fn update_rotation(&mut self, mouse_delta: DVec2, sensitivity: f64) {
        let yaw_delta = -mouse_delta.x * sensitivity;
        let pitch_delta = mouse_delta.y * sensitivity;

        self.yaw += yaw_delta;
        self.pitch += pitch_delta;
        self.pitch = self.pitch.clamp(-89.5, 89.5);

        self.rotation = (self.yaw_quat() * self.pitch_quat()).normalize();
    }

    fn update_position(&mut self, input: &Input, time: &Time) {
        let mut velocity = Vec3::ZERO;
        let forward = self.forward_xz();
        let right = self.right_xz();
        let up = Vec3::Y;

        if input.keys.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if input.keys.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if input.keys.pressed(KeyCode::KeyD) {
            velocity += right;
        }
        if input.keys.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if input.keys.pressed(KeyCode::Space) {
            velocity += up;
        }
        if input.keys.pressed(KeyCode::ShiftLeft) {
            velocity -= up;
        }

        velocity = velocity.normalize_or_zero();
        self.position += velocity * self.movement_speed * time.delta().as_secs_f32();
    }

    fn get_rotation_from_view_vector(pos: Vec3, target: Vec3) -> (Quat, f64, f64) {
        let forward = (target - pos).normalize();
        let right = Vec3::Y.cross(forward).normalize();
        let up = forward.cross(right);

        let matrix = Mat3::from_cols(right, up, forward);
        let rotation = Quat::from_mat3(&matrix);

        let yaw = ((forward.z).atan2(forward.x) as f64).to_degrees();
        let pitch = ((forward.y).asin() as f64).to_degrees();

        (rotation, yaw, pitch)
    }

    pub fn init(mut commands: Commands, renderer_viewport: Res<RendererViewport>) {
        commands.insert_resource(Camera::new(
            Vec3::ZERO,
            Vec3::Z,
            45.0,
            renderer_viewport.get_size(),
            1.0,
            100.0,
            10.0,
        ));
    }

    pub fn update(
        mut camera: ResMut<Camera>,
        input: Res<Input>,
        time: Res<Time>,
        mut mouse_motion_events: EventReader<MouseMotion>,
    ) {
        camera.update_position(&input, &time);

        let mouse_delta: DVec2 = mouse_motion_events.read().map(|e| **e).sum();
        camera.update_rotation(mouse_delta, 0.1);
    }

    pub fn on_resize(mut camera: ResMut<Camera>, renderer_viewport: Res<RendererViewport>) {
        camera.reconfigure_aspect(renderer_viewport.get_size());
    }
}
