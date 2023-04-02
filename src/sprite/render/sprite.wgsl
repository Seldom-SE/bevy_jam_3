#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

#import bevy_render::view

@group(0) @binding(0)
var<uniform> view: View;

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @location(1) world_position: vec4<f32>,
#ifdef COLORED
    @location(2) color: vec4<f32>,
#endif
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_uv: vec2<f32>,
#ifdef COLORED
    @location(2) vertex_color: vec4<f32>,
#endif
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.world_position = vec4<f32>(vertex_position, 1.0);
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
#ifdef COLORED
    out.color = vertex_color;
#endif
    return out;
}

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

struct Light {
    pos: vec2<f32>,
    color: vec3<f32>,
    falloff: f32,
};
struct Lights {
    sky_light: vec3<f32>,
    point_light_count: u32,
    point_lights: array<Light, 256u>,
};
@group(2) @binding(0)
var<uniform> lights: Lights;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(sprite_texture, sprite_sampler, in.uv);
#ifdef COLORED
    color = in.color * color;
#endif

    var light = lights.sky_light;
    for (var i = u32(0); i < lights.point_light_count; i = i + u32(1)) {
        let d = lights.point_lights[i].pos - in.world_position.xy;
        let dist = dot(d, d);
        light = light + lights.point_lights[i].color * pow((1.0 / dist), lights.point_lights[i].falloff);
    }
    color = color * vec4<f32>(light, 1.0);

#ifdef TONEMAP_IN_SHADER
    color = tone_mapping(color);
#endif

    return color;
}