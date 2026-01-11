use geo::Point;

use crate::shape::compiled::shader::argument::COORD_SCALE;

/// A flattened BVH node for GPU consumption.
/// Layout: [min_lat, max_lat, min_lon, max_lon, left_first, right_child, count]
/// - For internal nodes: left_first = left child index, right_child = right child index, count = 0
/// - For leaf nodes: left_first = first point index, right_child = 0, count = number of points
#[derive(Debug, Clone, Copy)]
pub struct BvhNode {
    pub min_lat: i32,
    pub max_lat: i32,
    pub min_lon: i32,
    pub max_lon: i32,
    pub left_first: u32,
    pub right_child: u32,
    pub count: u32,
}

impl BvhNode {
    pub const SIZE_U32: usize = 7;

    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.min_lat.to_le_bytes());
        buffer.extend_from_slice(&self.max_lat.to_le_bytes());
        buffer.extend_from_slice(&self.min_lon.to_le_bytes());
        buffer.extend_from_slice(&self.max_lon.to_le_bytes());
        buffer.extend_from_slice(&self.left_first.to_le_bytes());
        buffer.extend_from_slice(&self.right_child.to_le_bytes());
        buffer.extend_from_slice(&self.count.to_le_bytes());
    }
}

/// A BVH (Bounding Volume Hierarchy) for efficient point cloud queries.
#[derive(Debug, Clone)]
pub struct PointBvh {
    pub nodes: Vec<BvhNode>,
    pub points: Vec<(i32, i32)>, // (lon, lat) in scaled coordinates
}

impl PointBvh {
    const MAX_LEAF_SIZE: usize = 8;

    /// Build a BVH from a collection of points.
    pub fn build(points: &[Point]) -> Self {
        if points.is_empty() {
            return Self {
                nodes: vec![BvhNode {
                    min_lat: 0,
                    max_lat: 0,
                    min_lon: 0,
                    max_lon: 0,
                    left_first: 0,
                    right_child: 0,
                    count: 0,
                }],
                points: vec![],
            };
        }

        // Convert points to scaled integer coordinates
        let scaled_points: Vec<(i32, i32)> = points
            .iter()
            .map(|p| {
                let lon = (p.x() * COORD_SCALE as f64).round() as i32;
                let lat = (p.y() * COORD_SCALE as f64).round() as i32;
                (lon, lat)
            })
            .collect();

        let mut nodes = Vec::new();
        let mut indices: Vec<usize> = (0..scaled_points.len()).collect();

        Self::build_recursive(&mut nodes, &mut indices, &scaled_points, 0, scaled_points.len());

        // Reorder points according to the BVH traversal order
        let reordered_points: Vec<(i32, i32)> = indices.iter().map(|&i| scaled_points[i]).collect();

        Self {
            nodes,
            points: reordered_points,
        }
    }

    fn compute_bounds(indices: &[usize], points: &[(i32, i32)], start: usize, end: usize) -> (i32, i32, i32, i32) {
        let mut min_lat = i32::MAX;
        let mut max_lat = i32::MIN;
        let mut min_lon = i32::MAX;
        let mut max_lon = i32::MIN;

        for i in start..end {
            let (lon, lat) = points[indices[i]];
            min_lat = min_lat.min(lat);
            max_lat = max_lat.max(lat);
            min_lon = min_lon.min(lon);
            max_lon = max_lon.max(lon);
        }

        (min_lat, max_lat, min_lon, max_lon)
    }

    fn build_recursive(
        nodes: &mut Vec<BvhNode>,
        indices: &mut [usize],
        points: &[(i32, i32)],
        start: usize,
        end: usize,
    ) -> usize {
        let (min_lat, max_lat, min_lon, max_lon) = Self::compute_bounds(indices, points, start, end);
        let count = end - start;

        let node_index = nodes.len();

        if count <= Self::MAX_LEAF_SIZE {
            // Create leaf node
            nodes.push(BvhNode {
                min_lat,
                max_lat,
                min_lon,
                max_lon,
                left_first: start as u32,
                right_child: 0,
                count: count as u32,
            });
            return node_index;
        }

        // Choose split axis (longest extent)
        let lat_extent = max_lat - min_lat;
        let lon_extent = max_lon - min_lon;
        let split_on_lat = lat_extent > lon_extent;

        // Sort indices by the chosen axis
        if split_on_lat {
            indices[start..end].sort_by_key(|&i| points[i].1);
        } else {
            indices[start..end].sort_by_key(|&i| points[i].0);
        }

        let mid = start + count / 2;

        // Reserve space for this node (we'll fill it in after children are built)
        nodes.push(BvhNode {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
            left_first: 0,
            right_child: 0,
            count: 0,
        });

        // Build children
        let left_child = Self::build_recursive(nodes, indices, points, start, mid);
        let right_child = Self::build_recursive(nodes, indices, points, mid, end);

        // Update this node with child indices
        nodes[node_index].left_first = left_child as u32;
        nodes[node_index].right_child = right_child as u32;

        node_index
    }

    /// Total size in u32 elements when serialized.
    pub fn serialized_size_u32(&self) -> usize {
        // nodes (7 u32 per node) + points (2 u32 per point) + header (2 u32: node_count, point_count)
        2 + self.nodes.len() * BvhNode::SIZE_U32 + self.points.len() * 2
    }

    /// Write the BVH to a buffer for GPU consumption.
    /// Layout: [node_count, point_count, nodes..., points...]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) {
        // Header
        buffer.extend_from_slice(&(self.nodes.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(self.points.len() as u32).to_le_bytes());

        // Nodes
        for node in &self.nodes {
            node.write_to_buffer(buffer);
        }

        // Points (lon, lat pairs)
        for &(lon, lat) in &self.points {
            buffer.extend_from_slice(&lon.to_le_bytes());
            buffer.extend_from_slice(&lat.to_le_bytes());
        }
    }
}
