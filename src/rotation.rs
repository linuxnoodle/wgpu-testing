use cgmath::SquareMatrix;
use winit::event::{KeyboardInput, VirtualKeyCode, ElementState, WindowEvent};

pub struct RotationDeg {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl RotationDeg {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }

    pub fn build_rotation_matrix(&self) -> cgmath::Matrix4<f32> {
        let yaw = cgmath::Matrix4::from_angle_y(cgmath::Deg(self.yaw));
        let pitch = cgmath::Matrix4::from_angle_x(cgmath::Deg(self.pitch));
        let roll = cgmath::Matrix4::from_angle_z(cgmath::Deg(self.roll));
        return yaw * pitch * roll;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RotationUniform{
    rotation: [[f32; 4]; 4],
}

impl RotationUniform {
    pub fn new() -> Self {
        Self {
            rotation: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_rotation(&mut self, rotation: &RotationDeg) {
        self.rotation = rotation.build_rotation_matrix().into();
    }
}
pub struct RotationController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl RotationController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
   pub fn update_matrix(&mut self, rotation: &mut RotationDeg, delta_time: f32) {
        if self.is_forward_pressed {
            rotation.pitch += self.speed * delta_time;
        }
        if self.is_backward_pressed {
            rotation.pitch -= self.speed * delta_time;
        }
        if self.is_left_pressed {
            rotation.yaw += self.speed * delta_time;
        }
        if self.is_right_pressed {
            rotation.yaw -= self.speed * delta_time;
        }
    } 
}
