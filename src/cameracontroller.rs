use std::time::Duration;
use cgmath::{InnerSpace, Rad, Vector3};
use num_traits::clamp;
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseScrollDelta}, keyboard::KeyCode};
use crate::camera::{self, Camera, SAFE_FRAC_PI_2};

#[derive(Debug)]
pub struct CameraController {
    // Changed amounts to use -1.0/1.0 for direction instead of 100.0
    move_forward: f32,
    move_backward: f32,
    move_left: f32,
    move_right: f32,
    move_up: f32,
    move_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            move_forward: 5.0,
            move_backward: 5.0,
            move_left: 0.0,
            move_right: 0.0,
            move_up: 0.0,
            move_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        // Use 1.0/-1.0 for direction instead of 100.0
        let amount = if state == ElementState::Pressed { 100.0 } else { 0.0 };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.move_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.move_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.move_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.move_right = amount;
                true
            }
            KeyCode::Space => {
                self.move_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.move_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        // Accumulate mouse input deltas
        self.rotate_horizontal += mouse_dx as f32;
        self.rotate_vertical += mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll += -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32 * 0.1,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Calculate movement vectors
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        // Calculate movement amounts using frame-time scaling
        let forward_amount = (self.move_forward - self.move_backward) * self.speed * dt;
        let right_amount = (self.move_right - self.move_left) * self.speed * dt;
        let vertical_amount = (self.move_up - self.move_down) * self.speed * dt;

        // Apply translations
        camera.position += forward * forward_amount;
        camera.position += right * right_amount;
        camera.position.y += vertical_amount;

        // Handle zoom/scroll with frame-time scaling
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Calculate rotations with frame-time scaling
        let rotate_horizontal = Rad(self.rotate_horizontal) * self.sensitivity * dt;
        let rotate_vertical = Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // Apply rotations
        camera.yaw += rotate_horizontal;
        camera.pitch += rotate_vertical;

        // Reset accumulated rotation values
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        camera.pitch = clamp(
            camera.pitch,
            -Rad(SAFE_FRAC_PI_2),
            Rad(SAFE_FRAC_PI_2)
        );
    }
}