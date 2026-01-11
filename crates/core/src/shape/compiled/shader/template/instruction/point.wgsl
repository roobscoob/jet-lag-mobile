#define_import_path template::instruction::point

#import template::arguments::{
    tile_bounds, popArgument,
    argument_read_i32
}
#import template::constants::{
    USE_ELLIPSOID, USE_HIGH_PRECISION, COORD_SCALE
}
#import template::distance::{
    haversine_distance, vincenty_distance
}

// Instruction: Point
fn point(sample: vec2<f32>, idx_ptr: ptr<function, u32>) -> i32 {
    let argument = popArgument(idx_ptr);
    let x = argument_read_i32(argument, 0u);
    let y = argument_read_i32(argument, 1u);

    var distance_m: f32;

    if (USE_HIGH_PRECISION) {
        distance_m = 0;
    } else {
        // f32 path
        let sample_lon_scaled = f32(tile_bounds.min_lon_deg) + sample.x * f32(tile_bounds.lon_span_deg);
        let sample_lat_scaled = f32(tile_bounds.min_lat_deg + tile_bounds.lat_span_deg) - sample.y * f32(tile_bounds.lat_span_deg);

        if (USE_ELLIPSOID) {
            distance_m = vincenty_distance(
                sample_lat_scaled / f32(COORD_SCALE),
                sample_lon_scaled / f32(COORD_SCALE),
                f32(y) / f32(COORD_SCALE),
                f32(x) / f32(COORD_SCALE)
            );
        } else {
            distance_m = haversine_distance(
                sample_lat_scaled / f32(COORD_SCALE),
                sample_lon_scaled / f32(COORD_SCALE),
                f32(y) / f32(COORD_SCALE),
                f32(x) / f32(COORD_SCALE)
            );
        }
    }

    return i32(distance_m * 100.0);
}