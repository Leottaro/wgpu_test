use cgmath::prelude::*;

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
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }
    }

    pub fn test_instances(size: u32, dimension: u32, centered: bool, spaced: f32) -> Vec<Self> {
        let x_range = (if dimension >= 1 { 0..size } else { 0..1 }).into_iter();
        let z_range = (if dimension >= 2 { 0..size } else { 0..1 }).into_iter();
        let y_range = (if dimension >= 3 { 0..size } else { 0..1 }).into_iter();
        let centered_vec = cgmath::Vector3::new(size as f32, size as f32, size as f32) / 2.0;
        x_range
            .flat_map(|x| {
                let z_range = z_range.clone();
                let y_range = y_range.clone();
                z_range.flat_map(move |z| {
                    y_range.clone().map(move |y| {
                        let mut position = cgmath::Vector3 {
                            x: x as f32,
                            y: y as f32,
                            z: z as f32,
                        };
                        if centered {
                            position -= centered_vec;
                        }
                        position *= spaced;

                        let rotation = cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        );

                        let scale = 1.0;

                        Self {
                            position,
                            rotation,
                            scale,
                        }
                    })
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn default_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let instance_data = vec![Instance::default_instance().raw()];
        let buffer_desc = wgpu::util::BufferInitDescriptor {
            label: Some("Default Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        };
        wgpu::util::DeviceExt::create_buffer_init(device, &buffer_desc)
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
