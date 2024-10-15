use super::bind_group;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn get_rgba(filename: &str) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let mut filepath = std::env::current_dir().unwrap();
        filepath.push(filename);
        let filepath = filepath.into_os_string().into_string().unwrap();

        let bytes = std::fs::read(&filepath).expect(&format!("cannot load texture: {}", filepath));
        let dynamic = image::load_from_memory(&bytes).unwrap();
        let rgba = dynamic.to_rgba8();
        rgba
    }

    pub fn new(
        filename: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let diffuse_rgba = Texture::get_rgba("img/stone.png");
        let dimensions = diffuse_rgba.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(&filename),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        };
        let texture = device.create_texture(&texture_descriptor);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(dimensions.0 * 4),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler_descriptor = wgpu::SamplerDescriptor {
            label: Some(filename),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        };
        let sampler = device.create_sampler(&sampler_descriptor);

        let bind_group = {
            let mut builder = bind_group::Builder::new(device);
            builder.set_layout(layout);
            builder.add_texture(&view, &sampler);
            builder.build(filename)
        };

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }
}
