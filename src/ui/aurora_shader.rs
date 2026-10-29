use eframe::egui_wgpu;
use eframe::epaint;
use eframe::wgpu;

const COSMIC_DUST_WGSL: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

struct Uniforms {
    time: f32,
    aspect: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Procedural floating cosmic dust layer with smooth neighbor search
fn dust_layer(uv: vec2<f32>, scale: f32, speed: f32, brightness: f32, t: f32) -> vec3<f32> {
    let p = uv * scale;
    let id = floor(p);
    let gv = fract(p) - 0.5;

    var color = vec3<f32>(0.0);

    for (var y = -1.0; y <= 1.0; y = y + 1.0) {
        for (var x = -1.0; x <= 1.0; x = x + 1.0) {
            let offset = vec2<f32>(x, y);
            let cell_id = id + offset;
            let rnd = hash22(cell_id);

            // Gentle drift upwards with smooth organic horizontal sway
            let drift_y = fract(rnd.y - t * speed * (0.55 + rnd.x * 0.45));
            let sway_x = sin(t * 0.65 + rnd.x * 6.28) * 0.16;

            let particle_pos = offset + (vec2<f32>(rnd.x, drift_y) - 0.5) + vec2<f32>(sway_x, 0.0);
            let d = length(gv - particle_pos);

            let size = 0.085 + rnd.x * 0.070;

            // Fast early-out: particles outside visible radius (0.28) contribute zero light
            if (d < 0.28) {
                // Silk-smooth lifecycle fade-in and fade-out
                let life_fade = sin(drift_y * 3.14159265);
                let life_smooth = life_fade * life_fade;

                // Gentle continuous breathing twinkle
                let twinkle = (sin(t * 1.2 + rnd.y * 6.28) * 0.20 + 0.80) * life_smooth;

                let dist_sq = d * d;
                let inv_s2 = 1.0 / (size * size);
                let core = exp(-dist_sq * (280.0 * inv_s2)) * 1.45 * twinkle;
                let halo = max(0.0, 1.0 - d * (3.5 / size)) * 0.35 * twinkle;
                let white_core = exp(-dist_sq * (900.0 * inv_s2)) * 0.40 * twinkle;

                // Deep elegant high-contrast blue palette (Electric Sapphire to Crystal Azure)
                let mote_color = mix(
                    vec3<f32>(0.20, 0.48, 0.95), // Vibrant Electric Sapphire
                    vec3<f32>(0.35, 0.78, 1.00), // Luminous Crystal Azure
                    rnd.y
                );

                color = color + (mote_color * (core + halo) + vec3<f32>(0.75, 0.88, 1.00) * white_core) * brightness;
            }
        }
    }
    return color;
}

// Organic Liquid Mesh Gradient background with rich undulating blue shades
fn mesh_gradient(uv: vec2<f32>, t: f32) -> vec3<f32> {
    let slow_t = t * 0.25;

    // 1. Fluid domain warping - cria as curvas, dobras e movimentação líquida do mesh
    var q = uv;
    q.x = q.x + sin(uv.y * 2.8 + slow_t * 0.9) * 0.26 + cos(uv.x * 2.2 - slow_t * 0.7) * 0.16;
    q.y = q.y + cos(uv.x * 2.5 - slow_t * 0.8) * 0.26 + sin(uv.y * 2.4 + slow_t * 0.6) * 0.16;

    // Segunda camada de warp para dar textura e complexidade fluida
    let q2 = vec2<f32>(
        sin(q.y * 3.2 - slow_t * 0.5) * 0.12,
        cos(q.x * 3.0 + slow_t * 0.5) * 0.12
    );
    let p_warp = q + q2;

    // 2. Centros móveis do Mesh Gradient que cruzam a tela suavemente
    let c1 = vec2<f32>(
        0.28 + 0.30 * sin(slow_t * 0.65 + 0.3),
        0.32 + 0.25 * cos(slow_t * 0.80 + 1.2)
    );
    let c2 = vec2<f32>(
        0.75 + 0.25 * cos(slow_t * 0.75 + 2.1),
        0.28 + 0.28 * sin(slow_t * 0.55 + 0.8)
    );
    let c3 = vec2<f32>(
        0.32 + 0.28 * cos(slow_t * 0.45 + 3.4),
        0.78 + 0.22 * sin(slow_t * 0.70 + 2.6)
    );
    let c4 = vec2<f32>(
        0.78 + 0.24 * sin(slow_t * 0.85 + 4.3),
        0.72 + 0.26 * cos(slow_t * 0.60 + 3.2)
    );
    let c5 = vec2<f32>(
        0.50 + 0.22 * sin(slow_t * 1.05 + 1.7),
        0.50 + 0.22 * cos(slow_t * 0.90 + 5.0)
    );

    // 3. Paleta rica em nuances de azul claramente visíveis, porém refinadas e mais escuras
    let col_base = vec3<f32>(0.008, 0.018, 0.045);      // Dark Midnight Velvet Base
    let col_sapphire = vec3<f32>(0.042, 0.170, 0.420);  // Deep Sapphire
    let col_cobalt = vec3<f32>(0.060, 0.230, 0.520);    // Luminous Nocturnal Cobalt
    let col_ocean = vec3<f32>(0.025, 0.180, 0.330);     // Oceanic Teal-Blue
    let col_indigo = vec3<f32>(0.070, 0.130, 0.400);    // Electric Night Indigo
    let col_royal = vec3<f32>(0.038, 0.140, 0.350);     // Royal Velvet Blue

    // 4. Intensidade de cada mancha com queda suave
    let d1 = length(p_warp - c1);
    let d2 = length(p_warp - c2);
    let d3 = length(p_warp - c3);
    let d4 = length(p_warp - c4);
    let d5 = length(p_warp - c5);

    let b1 = smoothstep(0.95, 0.08, d1);
    let b2 = smoothstep(0.90, 0.08, d2);
    let b3 = smoothstep(0.85, 0.08, d3);
    let b4 = smoothstep(0.92, 0.08, d4);
    let b5 = smoothstep(0.80, 0.08, d5);

    // Mesclagem orgânica em camadas sobre a base escura
    var bg = col_base;
    bg = mix(bg, col_sapphire, b1 * 0.65);
    bg = mix(bg, col_cobalt, b2 * 0.60);
    bg = mix(bg, col_ocean, b3 * 0.55);
    bg = mix(bg, col_indigo, b4 * 0.55);
    bg = mix(bg, col_royal, b5 * 0.60);

    // 5. Linhas e dobras de relevo fluido (efeito de luz e sombra ondulante)
    let wave = sin(p_warp.x * 3.5 + p_warp.y * 3.0 + slow_t * 1.2) * cos(p_warp.y * 2.8 - slow_t * 0.8);
    bg = bg * (0.80 + 0.25 * (wave * 0.5 + 0.5));

    return bg;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let aspect = uniforms.aspect;
    let p = vec2<f32>(uv.x * aspect, uv.y);

    // Keep time bounded to 200*PI to prevent float precision drift / NaN over long sessions
    let t = uniforms.time % 628.31853;

    // --- 1. DYNAMIC BLUE MESH GRADIENT BACKGROUND ---
    var bg = mesh_gradient(uv, t);

    // --- 2. OPTIMIZED PARALLAX FLOATING DUST PARTICLES ---
    // Layer 1: Main floating motes with crisp contrast & gentle drift
    let dust_mid = dust_layer(p + vec2<f32>(2.4, 5.1), 4.2, 0.055, 1.05, t);

    // Layer 2: Foreground luminous bokeh orbs (large, smooth particles)
    let dust_near = dust_layer(p + vec2<f32>(7.2, 3.8), 2.2, 0.085, 0.75, t);

    var final_col = bg + dust_mid + dust_near;

    // --- 3. EDGE VIGNETTE ---
    let center_dist = length(uv - vec2<f32>(0.5, 0.5));
    let vignette = smoothstep(1.30, 0.40, center_dist);
    final_col = final_col * (0.90 + 0.10 * vignette);

    // --- 4. DITHER ---
    let dither = (hash12(in.uv * 1200.0 + fract(t)) - 0.5) * (1.0 / 220.0);
    final_col = final_col + vec3<f32>(dither);

    return vec4<f32>(final_col, 1.0);
}
"#;

pub struct CosmicDustBackgroundRenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl CosmicDustBackgroundRenderState {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cosmic_dust_background_shader"),
            source: wgpu::ShaderSource::Wgsl(COSMIC_DUST_WGSL.into()),
        });

        // Safe D3D12 / Vulkan constant buffer minimum alignment (256 bytes)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cosmic_dust_uniforms"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cosmic_dust_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cosmic_dust_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cosmic_dust_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cosmic_dust_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
        }
    }
}

pub struct CosmicDustCallback {
    pub time: f32,
    pub aspect: f32,
}

impl egui_wgpu::CallbackTrait for CosmicDustCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(state) = callback_resources.get::<CosmicDustBackgroundRenderState>() {
            let mut raw_bytes = [0u8; 256];
            let uniforms: [f32; 4] = [self.time, self.aspect, 0.0, 0.0];
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    uniforms.as_ptr() as *const u8,
                    std::mem::size_of_val(&uniforms),
                )
            };
            raw_bytes[..bytes.len()].copy_from_slice(bytes);
            queue.write_buffer(&state.uniform_buffer, 0, &raw_bytes);
        }
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        if let Some(state) = callback_resources.get::<CosmicDustBackgroundRenderState>() {
            render_pass.set_pipeline(&state.pipeline);
            render_pass.set_bind_group(0, &state.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
