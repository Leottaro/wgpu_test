use glfw::{fail_on_errors, Action, Context, Key, Window};
mod renderer_backend;
use cgmath::prelude::*;
use renderer_backend::{
    bind_group, bind_group_layout, camera, instance, mesh_builder, pipeline, texture::Texture,
};
use wgpu::util::DeviceExt;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32,
    0.0,
    NUM_INSTANCES_PER_ROW as f32,
);

struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut Window,
    render_pipeline: wgpu::RenderPipeline,
    mesh: mesh_builder::Mesh,
    face_texture: Texture,
    camera: camera::Camera,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_controller: camera::CameraController,
    instances: Vec<instance::Instance>,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut Window) -> Self {
        let size = window.get_framebuffer_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);

        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }.unwrap();
        let surface = unsafe { instance.create_surface_unsafe(target) }.unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|format| format.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = camera::Camera {
            eye: (-2.0, 1.0, 0.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_buffer(wgpu::ShaderStages::VERTEX);
            builder.build("Camera Bind Group Layout")
        };

        let texture_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_texture();
            builder.build("Texture Bind Group Layout")
        };

        let render_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            builder.add_vertex_buffer_layout(mesh_builder::Vertex::desc());
            builder.add_vertex_buffer_layout(instance::InstanceRaw::desc());
            builder.set_shader_module("shaders/shader.wgsl", "vertex_main", "fragment_main");
            builder.set_pixel_format(config.format);
            builder.add_bind_group_layout(&camera_bind_group_layout);
            builder.add_bind_group_layout(&texture_bind_group_layout);
            builder.build_pipeline("Render Pipeline")
        };

        let mesh = mesh_builder::make_cube(&device);

        let face_texture =
            Texture::new("img/stone.png", &device, &queue, &texture_bind_group_layout);

        let camera_controller = camera::CameraController::new(0.2);

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    instance::Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        Self {
            instance,
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            mesh,
            face_texture,
            camera,
            camera_bind_group_layout,
            camera_controller,
            instances,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Renderpass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);

        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = {
            let mut builder = bind_group::Builder::new(&self.device);
            builder.add_buffer(&camera_buffer);
            builder.set_layout(&self.camera_bind_group_layout);
            builder.build("Camera Bind Group")
        };

        let instance_data = self
            .instances
            .iter()
            .map(instance::Instance::raw)
            .collect::<Vec<_>>();
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.face_texture.bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(
                0..(self.mesh.index_buffer.size() as u32 / 2),
                0,
                0..self.instances.len() as u32,
            );
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();

        Ok(())
    }

    fn resize(&mut self, size: (i32, i32)) {
        if size.0 <= 0 || size.1 <= 0 {
            return;
        }
        self.size = size;
        self.config.width = size.0 as u32;
        self.config.height = size.1 as u32;
        self.surface.configure(&self.device, &self.config);
    }

    fn update_surface(&mut self, size: Option<(i32, i32)>) {
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&self.window) }.unwrap();
        self.surface = unsafe { self.instance.create_surface_unsafe(target) }.unwrap();
        if size.is_some() {
            self.resize(size.unwrap());
        } else {
            self.surface.configure(&self.device, &self.config);
        }
    }
}

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    let (mut window, events) = glfw
        .create_window(1280, 720, "GPU time !", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_all_polling(true);
    window.make_current();

    let mut state = State::new(&mut window).await;

    while !state.window.should_close() {
        state.camera_controller.update_camera(&mut state.camera);
        // println!("Camera: {:?}", state.camera.eye);

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            if state.camera_controller.process_events(&event) {
                continue;
            }

            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    println!("Escape pressed: closing window...");
                    state.window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(witdh, height) => {
                    state.update_surface(Some((witdh, height)));
                }
                glfw::WindowEvent::Pos(..) => {
                    state.update_surface(None);
                }
                _ => {
                    // println!("Event: {:?}", event);
                }
            }
        }

        match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.update_surface(None);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
        state.window.swap_buffers();
    }
}

fn main() {
    pollster::block_on(run());
}
