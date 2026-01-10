#define_import_path template

#import template::constants::{
    USE_ELLIPSOID, USE_HIGH_PRECISION, COORD_SCALE
}
#import template::arguments::{
    ShaderArgument, TileBounds,
    tile_bounds, popArgument,
    argument_read_u32, argument_read_i32, argument_read_f32
}
#import template::distance::{
    haversine_distance, vincenty_distance
}

fn compute(sample: vec2<f32>, idx_ptr: ptr<function, u32>) -> i32 {
    // Placeholder - body is replaced by code generator at runtime
    return 0;
}

@fragment
fn main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4f {
    // frag_coord is in pixel coordinates [0, 256)
    // Convert to [0,1] tile-local coordinates
    let sample = frag_coord.xy / 256.0;

    // Call point with our sample
    var arg_idx = 0u;
    let distance_cm_i32 = compute(sample, &arg_idx);
    let distance_cm = bitcast<u32>(distance_cm_i32);

    return vec4f(
        f32((distance_cm & 0x000000FF) >> 0) / 255.0,
        f32((distance_cm & 0x0000FF00) >> 8) / 255.0,
        f32((distance_cm & 0x00FF0000) >> 16) / 255.0,
        f32((distance_cm & 0xFF000000) >> 24) / 255.0,
    );
}
