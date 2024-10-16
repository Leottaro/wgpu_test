use cgmath::prelude::*;
use std::ops::*;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub position: [f32; 3],
    pub scale: f32,
}

const NUM_INSTANCES_PER_ROW: u32 = 20;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 / 2.0,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 / 2.0,
);

impl Instance {
    pub fn raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
            position: self.position.into(),
            scale: self.scale,
        }
    }

    pub fn default_instance() -> Self {
        Self {
            position: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }
    }

    pub fn test_instances() -> Vec<Self> {
        (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let mut position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    };
                    position -= INSTANCE_DISPLACEMENT;
                    position *= 2.0;
                    position += cgmath::Vector3::new(-0.5, -0.5, -0.5);

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    let scale = f32::cos(position.x as f32 * 0.75).powi(2).div(2.0)
                        * f32::cos(position.z as f32 * 0.75).powi(2).div(2.0)
                        + 1.0;

                    Self {
                        position,
                        rotation,
                        scale,
                    }
                })
            })
            .collect::<Vec<_>>()
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
