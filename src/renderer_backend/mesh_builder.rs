use cgmath::{Vector2, Vector3};
use wgpu::util::DeviceExt;

pub struct Vertex {
    position: Vector3<f32>,
    tex_coords: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Vertex {
    pub fn new(data: [f32; 5]) -> Self {
        Vertex {
            position: Vector3::new(data[0], data[1], data[2]),
            tex_coords: Vector2::new(data[3], data[4]),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    pub fn raw(&self) -> VertexRaw {
        VertexRaw {
            position: self.position.into(),
            tex_coords: self.tex_coords.into(),
        }
    }
}

pub fn make_cube(device: &wgpu::Device) -> Mesh {
    let vertices: [Vertex; 12] = [
        Vertex::new([0.0, 0.0, 0.0, 0.0, 1.0]),
        Vertex::new([0.0, 1.0, 0.0, 0.0, 0.0]),
        Vertex::new([0.0, 1.0, 1.0, 1.0, 0.0]),
        Vertex::new([0.0, 0.0, 1.0, 1.0, 1.0]),
        Vertex::new([1.0, 0.0, 0.0, 1.0, 1.0]),
        Vertex::new([1.0, 1.0, 0.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, 0.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0, 1.0, 1.0]),
        Vertex::new([1.0, 0.0, 0.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 1.0, 1.0, 0.0]),
    ];

    let vertices_uniform = vertices
        .into_iter()
        .map(|vertex| vertex.raw())
        .collect::<Vec<VertexRaw>>();
    let mut bytes = bytemuck::cast_slice(vertices_uniform.as_slice());

    let mut buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad Vertex Buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };
    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    let indices: [u16; 36] = [
        2, 1, 0, 0, 3, 2, // front
        4, 5, 6, 6, 7, 4, // back
        0, 1, 5, 5, 4, 0, // left
        6, 2, 3, 3, 7, 6, // right
        1, 2, 9, 9, 8, 1, // top
        11, 3, 0, 0, 10, 11, // bootom
    ];
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
