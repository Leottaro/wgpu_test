use cgmath::*;
use glfw::*;
use std::time::Duration;

use super::instance::Instance;

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - std::f32::EPSILON;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    front: Vector3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let yaw = yaw.into();
        let pitch = pitch.into();
        Self {
            position: position.into(),
            yaw,
            pitch,
            front: Vector3::zero(),
            forward: Vector3::zero(),
            right: Vector3::zero(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.position, self.front, Vector3::unit_y())
    }

    pub fn update_directions(&mut self) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        self.front = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        self.right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

struct Plane {
    normal: Vector3<f32>,
    distance: f32,
}

impl Plane {
    pub fn new_from_point(normal: Vector3<f32>, point: Point3<f32>) -> Self {
        let normal = normal.normalize();
        let distance = normal.dot(point.to_vec());
        Self { normal, distance }
    }

    pub fn get_signed_distance(&self, vector: Vector3<f32>) -> f32 {
        self.normal.dot(vector) - self.distance
    }
}

const CENTER_OFFSET: Vector3<f32> = Vector3::new(0.5, 0.5, 0.5);
pub struct Frustum {
    near_plane: Plane,
    far_plane: Plane,

    top_plane: Plane,
    bottom_plane: Plane,

    right_plane: Plane,
    left_plane: Plane,
}

impl Frustum {
    pub fn new(camera: &Camera, projection: &Projection) -> Self {
        let far_height: f32 = projection.zfar * f32::tan(projection.fovy.0 / 2.0);
        let far_width: f32 = far_height * projection.aspect;
        let camera_up = camera.front.cross(camera.right);

        let near_center = camera.position + camera.front * projection.znear;
        let near_plane = Plane::new_from_point(camera.front, near_center);

        let far_center = camera.position + camera.front * projection.zfar;
        let far_plane = Plane::new_from_point(-camera.front, far_center);

        let top_plane = Plane::new_from_point(
            -cgmath::Vector3::cross(
                camera.right,
                camera.front * projection.zfar - camera_up * far_height,
            ),
            camera.position,
        );
        let bottom_plane = Plane::new_from_point(
            -cgmath::Vector3::cross(
                camera.front * projection.zfar + camera_up * far_height,
                camera.right,
            ),
            camera.position,
        );
        let right_plane = Plane::new_from_point(
            -cgmath::Vector3::cross(
                camera.front * projection.zfar - camera.right * far_width,
                camera_up,
            ),
            camera.position,
        );
        let left_plane = Plane::new_from_point(
            -cgmath::Vector3::cross(
                camera_up,
                camera.front * projection.zfar + camera.right * far_width,
            ),
            camera.position,
        );

        Self {
            near_plane,
            far_plane,
            top_plane,
            bottom_plane,
            right_plane,
            left_plane,
        }
    }

    pub fn is_inside(&self, vector: Vector3<f32>) -> bool {
        let mut inside = true;
        let planes = vec![
            &self.near_plane,
            &self.far_plane,
            &self.top_plane,
            &self.bottom_plane,
            &self.right_plane,
            &self.left_plane,
        ];
        for plane in planes {
            if plane.normal.dot(vector) - plane.distance < 0.0 {
                inside = false;
                break;
            }
        }
        inside
    }

    pub fn is_inside_instance(&self, instance: &Instance) -> bool {
        let mut inside = true;
        let planes = vec![
            &self.near_plane,
            &self.far_plane,
            &self.top_plane,
            &self.bottom_plane,
            &self.right_plane,
            &self.left_plane,
        ];
        for plane in planes {
            let truc = plane.get_signed_distance(instance.position + CENTER_OFFSET);
            if truc < -instance.scale * f32::sqrt(3.0) / 2.0 {
                inside = false;
                break;
            }
        }
        inside
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0, 0.0, 0.0, 1.0],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    old_cursor_pos: (f64, f64),
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32, old_cursor_pos: (f64, f64)) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            old_cursor_pos,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Key(key, _, action, _) => self.process_keyboard(*key, *action),
            WindowEvent::CursorPos(x, y) => {
                self.process_mouse(x - self.old_cursor_pos.0, y - self.old_cursor_pos.1);
                self.old_cursor_pos = (*x, *y);
                true
            }
            WindowEvent::Scroll(_, y) => {
                self.process_scroll(*y as f32);
                true
            }
            _ => false,
        }
    }

    pub fn process_keyboard(&mut self, key: Key, state: Action) -> bool {
        let amount = if state == Action::Release { 0.0 } else { 1.0 };
        match key {
            Key::W | Key::Up => {
                self.amount_forward = amount;
                true
            }
            Key::S | Key::Down => {
                self.amount_backward = amount;
                true
            }
            Key::A | Key::Left => {
                self.amount_left = amount;
                true
            }
            Key::D | Key::Right => {
                self.amount_right = amount;
                true
            }
            Key::Space => {
                self.amount_up = amount;
                true
            }
            Key::LeftShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, scroll: f32) {
        self.scroll = -scroll;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // recalculates the camera's direction vectors
        camera.update_directions();

        // Move forward/backward and left/right
        camera.position +=
            camera.forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += camera.right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        camera.position += camera.front * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
