use cgmath::EuclideanSpace;
use glfw::WindowEvent;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub spherical_coords: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn to_cartesian_coords(&self) -> cgmath::Point3<f32> {
        let r = self.spherical_coords.x;
        let theta = self.spherical_coords.y;
        let phi = self.spherical_coords.z;
        cgmath::Point3 {
            x: r * f32::sin(theta) * f32::cos(phi),
            y: r * f32::cos(theta),
            z: r * f32::sin(theta) * f32::sin(phi),
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(
            self.to_cartesian_coords(),
            cgmath::Point3::origin(),
            self.up,
        );
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    zoom: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}
const THRESHOLD: f32 = 0.1;

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            zoom: 0.0,
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        self.zoom = 0.0;
        match event {
            WindowEvent::Key(key, _, action, _) => {
                if action == &glfw::Action::Repeat {
                    return false;
                }
                let is_pressed = *action == glfw::Action::Press;
                match key {
                    glfw::Key::Up | glfw::Key::W => self.is_up_pressed = is_pressed,
                    glfw::Key::Down | glfw::Key::S => self.is_down_pressed = is_pressed,
                    glfw::Key::Left | glfw::Key::A => self.is_left_pressed = is_pressed,
                    glfw::Key::Right | glfw::Key::D => self.is_right_pressed = is_pressed,
                    _ => return false,
                };
                true
            }
            WindowEvent::Scroll(_, y) => {
                if f64::abs(*y) > THRESHOLD as f64 {
                    self.zoom = *y as f32;
                } else {
                    self.zoom = 0.0;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use std::f32::consts::{PI, TAU};
        let (mut r, mut theta, mut phi) = camera.spherical_coords.into();
        let zoom_speed = self.zoom * self.speed;
        if r + zoom_speed > THRESHOLD {
            r += zoom_speed;
        } else {
            r = THRESHOLD;
        }

        let right_speed = (self.is_right_pressed as i8 - self.is_left_pressed as i8) as f32
            * (self.speed * TAU * 0.01);
        if right_speed != 0.0 {
            phi -= right_speed;
            while phi < 0.0 || phi > TAU {
                if phi > TAU {
                    phi -= TAU;
                } else if phi < 0.0 {
                    phi += TAU;
                }
            }
        }

        let up_speed = (self.is_up_pressed as i8 - self.is_down_pressed as i8) as f32
            * (self.speed * PI * 0.01);
        if up_speed != 0.0 {
            theta = f32::min(f32::max(theta - up_speed, THRESHOLD), PI - THRESHOLD);
        }

        camera.spherical_coords = cgmath::Point3::new(r, theta, phi);
    }
}
