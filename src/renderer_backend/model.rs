use wgpu::util::DeviceExt;

use super::{bind_group, instance, texture};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const MODEL_DIR: &str = "models";

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

fn load_string(file_name: &str) -> String {
    let mut filepath = std::env::current_dir().unwrap();
    filepath.push(crate::RESSOURCES_DIR);
    filepath.push(MODEL_DIR);
    filepath.push(file_name);
    let filepath = filepath.into_os_string().into_string().unwrap();
    std::fs::read_to_string(&filepath).expect(&format!("cannot load model: {}", filepath))
}

fn load_bytes(file_name: &str) -> Vec<u8> {
    let mut filepath = std::env::current_dir().unwrap();
    filepath.push(crate::RESSOURCES_DIR);
    filepath.push(MODEL_DIR);
    filepath.push(file_name);
    let filepath = filepath.into_os_string().into_string().unwrap();
    std::fs::read(&filepath).expect(&format!("cannot load model: {}", filepath))
}

impl Model {
    pub fn load_model(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let obj_text = load_string(file_name);
        let obj_cursor = std::io::Cursor::new(obj_text);
        let mut obj_reader = std::io::BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            move |filepath| {
                let mat_text = std::fs::read_to_string(filepath).expect(&format!(
                    "cannot load model: {}",
                    filepath.to_str().unwrap()
                ));
                tobj::load_mtl_buf(&mut std::io::BufReader::new(std::io::Cursor::new(mat_text)))
            },
        )
        .expect(&format!("couldn't read model file {}", file_name));

        let texture_layout = texture::Texture::default_layout(device);
        let materials = obj_materials
            .unwrap()
            .into_iter()
            .map(|m| {
                let diffuse_texture = {
                    let bytes = load_bytes(file_name);
                    let img = image::load_from_memory(&bytes).unwrap();
                    texture::Texture::from_image(&img, device, queue, None, Some(file_name))
                };

                let bind_group = {
                    let mut builder = bind_group::Builder::new(device);
                    builder.set_layout(&texture_layout);
                    builder.add_texture(&diffuse_texture.view, &diffuse_texture.sampler);
                    builder.build(&format!("{} bind group", file_name))
                };

                Material {
                    name: m.name,
                    diffuse_texture,
                    bind_group,
                }
            })
            .collect::<Vec<Material>>();

        let meshes = models
            .into_iter()
            .map(|model| {
                let verticies = (0..model.mesh.positions.len() / 3)
                    .map(move |i| {
                        if model.mesh.normals.is_empty() {
                            ModelVertex {
                                position: [
                                    model.mesh.positions[i * 3],
                                    model.mesh.positions[i * 3 + 1],
                                    model.mesh.positions[i * 3 + 2],
                                ],
                                tex_coords: [
                                    model.mesh.texcoords[i * 2],
                                    1.0 - model.mesh.texcoords[i * 2 + 1],
                                ],
                                normal: [0.0, 0.0, 0.0],
                            }
                        } else {
                            ModelVertex {
                                position: [
                                    model.mesh.positions[i * 3],
                                    model.mesh.positions[i * 3 + 1],
                                    model.mesh.positions[i * 3 + 2],
                                ],
                                tex_coords: [
                                    model.mesh.texcoords[i * 2],
                                    1.0 - model.mesh.texcoords[i * 2 + 1],
                                ],
                                normal: [
                                    model.mesh.normals[i * 3],
                                    model.mesh.normals[i * 3 + 1],
                                    model.mesh.normals[i * 3 + 2],
                                ],
                            }
                        }
                    })
                    .collect::<Vec<ModelVertex>>();

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verticies),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&model.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: model.mesh.indices.len() as u32,
                    material: model.mesh.material_id.unwrap_or(0),
                }
            })
            .collect::<Vec<Mesh>>();

        Model { meshes, materials }
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, device: &wgpu::Device);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: std::ops::Range<u32>,
        instance_buffer: &wgpu::Buffer,
    );
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, device: &wgpu::Device) {
        self.draw_mesh_instanced(mesh, 0..1, &instance::Instance::default_buffer(device));
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: std::ops::Range<u32>,
        instance_buffer: &wgpu::Buffer,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}
