#define_import_path template::arguments

struct ShaderArgument {
    offset: u32,
    length: u32,
}

struct TileBounds {
    min_lat_deg: i32,  // southwest corner
    min_lon_deg: i32,
    lat_span_deg: i32, // tile height in degrees
    lon_span_deg: i32, // tile width in degrees
}

@group(0) @binding(0)
var<storage, read> arguments: array<ShaderArgument>;

@group(0) @binding(1)
var<storage, read> argument_data: array<u32>;

@group(0) @binding(2)
var<uniform> tile_bounds: TileBounds;

fn popArgument(idx_ptr: ptr<function, u32>) -> ShaderArgument {
    let idx = *idx_ptr;
    *idx_ptr = idx + 1u;
    return arguments[idx];
}

fn argument_read_u32(argument: ShaderArgument, index: u32) -> u32 {
    return argument_data[argument.offset + index];
}

fn argument_read_i32(argument: ShaderArgument, index: u32) -> i32 {
    return bitcast<i32>(argument_data[argument.offset + index]);
}

fn argument_read_f32(argument: ShaderArgument, index: u32) -> f32 {
    return bitcast<f32>(argument_data[argument.offset + index]);
}
