#define_import_path template::instruction::point_cloud

#import template::arguments::{
    tile_bounds, popArgument, argument_data
}
#import template::constants::{
    USE_ELLIPSOID, USE_HIGH_PRECISION, COORD_SCALE
}
#import template::distance::{
    haversine_distance, vincenty_distance
}

// BVH Node structure (7 u32s per node):
// [min_lat, max_lat, min_lon, max_lon, left_first, right_child, count]
// - For internal nodes: count = 0, left_first = left child index, right_child = right child index
// - For leaf nodes: count > 0, left_first = first point index, right_child = 0
const BVH_NODE_SIZE: u32 = 7u;

// Maximum stack depth for BVH traversal (log2 of max points)
const MAX_STACK_DEPTH: u32 = 32u;

fn read_bvh_node_min_lat(argument_offset: u32, node_index: u32) -> i32 {
    // Skip header (2 u32s: node_count, point_count)
    return argument_read_i32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 0u);
}

fn read_bvh_node_max_lat(argument_offset: u32, node_index: u32) -> i32 {
    return argument_read_i32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 1u);
}

fn read_bvh_node_min_lon(argument_offset: u32, node_index: u32) -> i32 {
    return argument_read_i32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 2u);
}

fn read_bvh_node_max_lon(argument_offset: u32, node_index: u32) -> i32 {
    return argument_read_i32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 3u);
}

fn read_bvh_node_left_first(argument_offset: u32, node_index: u32) -> u32 {
    return argument_read_u32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 4u);
}

fn read_bvh_node_right_child(argument_offset: u32, node_index: u32) -> u32 {
    return argument_read_u32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 5u);
}

fn read_bvh_node_count(argument_offset: u32, node_index: u32) -> u32 {
    return argument_read_u32_offset(argument_offset + 2u + node_index * BVH_NODE_SIZE + 6u);
}

fn read_point_lon(argument_offset: u32, node_count: u32, point_index: u32) -> i32 {
    // Points start after header (2) and nodes (node_count * 7)
    let points_offset = argument_offset + 2u + node_count * BVH_NODE_SIZE;
    return argument_read_i32_offset(points_offset + point_index * 2u + 0u);
}

fn read_point_lat(argument_offset: u32, node_count: u32, point_index: u32) -> i32 {
    let points_offset = argument_offset + 2u + node_count * BVH_NODE_SIZE;
    return argument_read_i32_offset(points_offset + point_index * 2u + 1u);
}

// Helper to read from argument_data at absolute offset
fn argument_read_i32_offset(offset: u32) -> i32 {
    return bitcast<i32>(argument_data[offset]);
}

fn argument_read_u32_offset(offset: u32) -> u32 {
    return argument_data[offset];
}

// Compute distance from sample point to a point in the cloud
fn compute_point_distance(sample_lat: f32, sample_lon: f32, point_lat: f32, point_lon: f32) -> f32 {
    if (USE_ELLIPSOID) {
        return vincenty_distance(sample_lat, sample_lon, point_lat, point_lon);
    } else {
        return haversine_distance(sample_lat, sample_lon, point_lat, point_lon);
    }
}

// Cheap squared distance approximation for child ordering
// Uses flat-earth approximation - good enough for comparing two nearby bboxes
fn cheap_bbox_distance_sq(sample_lat: f32, sample_lon: f32, min_lat: f32, max_lat: f32, min_lon: f32, max_lon: f32) -> f32 {
    let closest_lat = clamp(sample_lat, min_lat, max_lat);
    let closest_lon = clamp(sample_lon, min_lon, max_lon);
    let dlat = sample_lat - closest_lat;
    let dlon = (sample_lon - closest_lon) * cos(sample_lat * DEG_TO_RAD);
    return dlat * dlat + dlon * dlon;
}

// Guaranteed lower-bound distance for bbox pruning
// Uses flat-earth approximation with safety factor - never overestimates actual geodesic distance
fn lower_bound_bbox_distance(sample_lat: f32, sample_lon: f32, min_lat: f32, max_lat: f32, min_lon: f32, max_lon: f32) -> f32 {
    let closest_lat = clamp(sample_lat, min_lat, max_lat);
    let closest_lon = clamp(sample_lon, min_lon, max_lon);

    // If sample is inside the box, distance is 0
    if (closest_lat == sample_lat && closest_lon == sample_lon) {
        return 0.0;
    }

    // Flat-earth approximation in meters
    let dlat_m = (sample_lat - closest_lat) * 110574.0;  // min meters/deg latitude
    let dlon_m = (sample_lon - closest_lon) * 111320.0 * cos(sample_lat * DEG_TO_RAD);
    let flat_dist = sqrt(dlat_m * dlat_m + dlon_m * dlon_m);

    // 1% safety margin ensures this is always <= actual geodesic distance
    return flat_dist * 0.99;
}

// Instruction: PointCloud with BVH traversal
fn point_cloud(sample: vec2<f32>, idx_ptr: ptr<function, u32>) -> i32 {
    let argument = popArgument(idx_ptr);
    let argument_offset = argument.offset;

    // Read BVH header
    let node_count = argument_read_u32_offset(argument_offset + 0u);
    let point_count = argument_read_u32_offset(argument_offset + 1u);

    if (point_count == 0u) {
        return 2147483647; // i32::MAX - no points
    }

    // Convert sample to lat/lon in degrees
    let sample_lon_scaled = f32(tile_bounds.min_lon_deg) + sample.x * f32(tile_bounds.lon_span_deg);
    let sample_lat_scaled = f32(tile_bounds.min_lat_deg + tile_bounds.lat_span_deg) - sample.y * f32(tile_bounds.lat_span_deg);
    let sample_lat = sample_lat_scaled / f32(COORD_SCALE);
    let sample_lon = sample_lon_scaled / f32(COORD_SCALE);

    var min_distance_m: f32 = 3.402823e+38; // f32::MAX

    // Stack for iterative BVH traversal
    var stack: array<u32, MAX_STACK_DEPTH>;
    var stack_ptr: u32 = 0u;

    // Start with root node
    stack[0] = 0u;
    stack_ptr = 1u;

    while (stack_ptr > 0u) {
        stack_ptr -= 1u;
        let node_index = stack[stack_ptr];

        // Read node bounding box
        let node_min_lat = f32(read_bvh_node_min_lat(argument_offset, node_index)) / f32(COORD_SCALE);
        let node_max_lat = f32(read_bvh_node_max_lat(argument_offset, node_index)) / f32(COORD_SCALE);
        let node_min_lon = f32(read_bvh_node_min_lon(argument_offset, node_index)) / f32(COORD_SCALE);
        let node_max_lon = f32(read_bvh_node_max_lon(argument_offset, node_index)) / f32(COORD_SCALE);

        // Early exit: if bbox lower-bound distance is farther than current best, skip
        // Uses cheap flat-earth approximation that never overestimates (guaranteed lower bound)
        let bbox_dist = lower_bound_bbox_distance(sample_lat, sample_lon, node_min_lat, node_max_lat, node_min_lon, node_max_lon);
        if (bbox_dist >= min_distance_m) {
            continue;
        }

        let left_first = read_bvh_node_left_first(argument_offset, node_index);
        let count = read_bvh_node_count(argument_offset, node_index);

        if (count > 0u) {
            // Leaf node: check all points
            for (var i: u32 = 0u; i < count; i += 1u) {
                let point_index = left_first + i;
                let point_lon = f32(read_point_lon(argument_offset, node_count, point_index)) / f32(COORD_SCALE);
                let point_lat = f32(read_point_lat(argument_offset, node_count, point_index)) / f32(COORD_SCALE);

                let dist = compute_point_distance(sample_lat, sample_lon, point_lat, point_lon);
                min_distance_m = min(min_distance_m, dist);
            }
        } else {
            // Internal node: push children in distance order (far first, near last)
            let left_child = left_first;
            let right_child = read_bvh_node_right_child(argument_offset, node_index);

            if (stack_ptr < MAX_STACK_DEPTH - 1u) {
                // Read child bboxes for ordering
                let left_min_lat = f32(read_bvh_node_min_lat(argument_offset, left_child)) / f32(COORD_SCALE);
                let left_max_lat = f32(read_bvh_node_max_lat(argument_offset, left_child)) / f32(COORD_SCALE);
                let left_min_lon = f32(read_bvh_node_min_lon(argument_offset, left_child)) / f32(COORD_SCALE);
                let left_max_lon = f32(read_bvh_node_max_lon(argument_offset, left_child)) / f32(COORD_SCALE);

                let right_min_lat = f32(read_bvh_node_min_lat(argument_offset, right_child)) / f32(COORD_SCALE);
                let right_max_lat = f32(read_bvh_node_max_lat(argument_offset, right_child)) / f32(COORD_SCALE);
                let right_min_lon = f32(read_bvh_node_min_lon(argument_offset, right_child)) / f32(COORD_SCALE);
                let right_max_lon = f32(read_bvh_node_max_lon(argument_offset, right_child)) / f32(COORD_SCALE);

                let left_dist = cheap_bbox_distance_sq(sample_lat, sample_lon, left_min_lat, left_max_lat, left_min_lon, left_max_lon);
                let right_dist = cheap_bbox_distance_sq(sample_lat, sample_lon, right_min_lat, right_max_lat, right_min_lon, right_max_lon);

                // Push farther child first (will be popped second), nearer child last (popped first)
                if (left_dist < right_dist) {
                    stack[stack_ptr] = right_child;
                    stack_ptr += 1u;
                    stack[stack_ptr] = left_child;
                    stack_ptr += 1u;
                } else {
                    stack[stack_ptr] = left_child;
                    stack_ptr += 1u;
                    stack[stack_ptr] = right_child;
                    stack_ptr += 1u;
                }
            }
        }
    }

    return i32(min_distance_m * 100.0);
}
