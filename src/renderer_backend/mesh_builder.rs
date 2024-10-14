use cgmath::{Vector2, Vector3};
use wgpu::util::DeviceExt;

pub struct Vertex {
    position: Vector3<f32>,
    tex_coords: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexUniform {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    pub fn to_uniform(&self) -> VertexUniform {
        VertexUniform {
            position: self.position.into(),
            tex_coords: self.tex_coords.into(),
        }
    }
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    let vertices: [Vertex; 4] = [
        Vertex {
            position: Vector3::new(-1.0, -1.0, 0.0),
            tex_coords: Vector2::new(0.0, 0.0),
        },
        Vertex {
            position: Vector3::new(1.0, -1.0, 0.0),
            tex_coords: Vector2::new(1.0, 0.0),
        },
        Vertex {
            position: Vector3::new(-1.0, 1.0, 0.0),
            tex_coords: Vector2::new(0.0, 1.0),
        },
        Vertex {
            position: Vector3::new(1.0, 1.0, 0.0),
            tex_coords: Vector2::new(1.0, 1.0),
        },
    ];
    let vertices_uniform = vertices
        .into_iter()
        .map(|vertex| vertex.to_uniform())
        .collect::<Vec<VertexUniform>>();
    let mut bytes = bytemuck::cast_slice(vertices_uniform.as_slice());

    let mut buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad Vertex Buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };
    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    let indices: [u16; 6] = [0, 1, 3, 3, 2, 0];
    bytes = bytemuck::cast_slice(&indices);

    buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad Index Buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::INDEX,
    };
    let index_buffer = device.create_buffer_init(&buffer_descriptor);

    Mesh {
        vertex_buffer,
        index_buffer,
    }
}
