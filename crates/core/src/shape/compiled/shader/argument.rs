use crate::{
    map::tile::Tile,
    shape::{bvh::PointBvh, compiled::shader::ShaderArgument, types::Centimeters},
};

pub trait IntoShaderArgument {
    fn into_shader_argument(&self, buffer: &mut Vec<u8>, tile: &Tile) -> Vec<ShaderArgument>;
}

pub const COORD_SCALE: i32 = 10_000_000;

impl IntoShaderArgument for geo::Point {
    fn into_shader_argument(&self, buffer: &mut Vec<u8>, _tile: &Tile) -> Vec<ShaderArgument> {
        // convert into (i32, i32) where each value is the f32 * COORD_SCALE
        let x = (self.x() * COORD_SCALE as f64).round() as i32;
        let y = (self.y() * COORD_SCALE as f64).round() as i32;

        // Offset in u32 indices (array<u32> has 4-byte elements)
        let offset = (buffer.len() / 4) as u32;

        buffer.extend_from_slice(&x.to_le_bytes());
        buffer.extend_from_slice(&y.to_le_bytes());

        vec![ShaderArgument { offset, length: 2 }]
    }
}

impl IntoShaderArgument for PointBvh {
    fn into_shader_argument(&self, buffer: &mut Vec<u8>, _tile: &Tile) -> Vec<ShaderArgument> {
        let offset = (buffer.len() / 4) as u32;
        let length = self.serialized_size_u32() as u32;

        self.write_to_buffer(buffer);

        vec![ShaderArgument { offset, length }]
    }
}

impl IntoShaderArgument for Centimeters {
    fn into_shader_argument(&self, buffer: &mut Vec<u8>, _tile: &Tile) -> Vec<ShaderArgument> {
        let offset = (buffer.len() / 4) as u32;

        let value_cm = self.0;
        buffer.extend_from_slice(&value_cm.to_le_bytes());

        vec![ShaderArgument { offset, length: 1 }]
    }
}
