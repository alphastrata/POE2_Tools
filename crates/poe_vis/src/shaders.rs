//!
//! Most of the boilerplate to make a custom shader work lives here.
//! You may notice this comes from github.com/alphastrata/shadplay well.. I like to reuse work :P
use bevy::{prelude::*, reflect::TypePath, render::render_resource::*, sprite::Material2d};
use std::path::PathBuf;

/// The 2D shadertoy like shader
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct YourShader2D {
    #[uniform(0)]
    pub(crate) mouse_pos: MousePos,

    #[texture(1, dimension = "2d")]
    #[sampler(2)]
    pub img: Handle<Image>,
}

#[derive(ShaderType, Debug, Clone)]
pub struct MousePos {
    pub x: f32,
    pub y: f32,
}

impl Material2d for YourShader2D {
    fn fragment_shader() -> ShaderRef {
        "shaders/myshader_2d.wgsl".into()
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct DottedLineShader {
    #[uniform(100)]
    uniforms: Holder, //RGBA
}

/// Simplified holding struct to make passing across uniform(n) simpler.
#[derive(ShaderType, Default, Clone, Debug)]
struct Holder {
    tint: LinearRgba,
    /// How wide do you want the line as a % of its availablu uv space: 0.5 would be 50% of the surface of the geometry
    line_width: f32,
    /// How many segments (transparent 'cuts') do you want?
    segments: f32,
    /// How fast do you want the animation to be? set 0.0 to disable.
    phase: f32,
    /// How far spaced apart do you want these lines?
    line_spacing: f32,
}

impl Material for DottedLineShader {
    fn fragment_shader() -> ShaderRef {
        "shaders/dotted_line.wgsl".into()
    }
}
