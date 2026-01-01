use anyhow::{Context, Result};
use clap::Parser;
use geo::{BooleanOps, Contains, Coord, Intersects, LineString, MultiPolygon, Polygon};
use geojson::{GeoJson, Value};
use prost::Message;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

mod vector_tile;

use vector_tile::tile::{GeomType, Layer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Input tile directory (z/x/y.mvt structure)"
    )]
    input: PathBuf,

    #[arg(short, long, help = "Output tile directory")]
    output: PathBuf,

    #[arg(short, long, help = "GeoJSON file containing MultiPolygon boundary")]
    geojson: PathBuf,

    #[arg(
        short,
        long,
        default_value = "4096",
        help = "Tile extent (default: 4096)"
    )]
    extent: u32,

    #[arg(long, help = "Input tiles are gzip compressed")]
    gzip: bool,
}

struct TileClipper {
    clip_polygon: MultiPolygon,
    extent: u32,
}

impl TileClipper {
    fn new(clip_polygon: MultiPolygon, extent: u32) -> Self {
        Self {
            clip_polygon,
            extent,
        }
    }

    fn clip_tile(&self, tile_data: &[u8], z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
        let tile = vector_tile::Tile::decode(tile_data).context("Failed to decode vector tile")?;

        let mut clipped_tile = vector_tile::Tile { layers: vec![] };

        for layer in tile.layers {
            if let Some(clipped_layer) = self.clip_layer(&layer, z, x, y)? {
                clipped_tile.layers.push(clipped_layer);
            }
        }

        if clipped_tile.layers.is_empty() {
            return Ok(None);
        }

        let mut buf = Vec::new();
        clipped_tile
            .encode(&mut buf)
            .context("Failed to encode clipped tile")?;

        Ok(Some(buf))
    }

    fn clip_layer(&self, layer: &Layer, z: u8, x: u32, y: u32) -> Result<Option<Layer>> {
        let extent = layer.extent.unwrap_or(4096);
        let tile_bounds = Self::tile_to_geo_bounds(z, x, y);

        let tile_clip_poly = self.get_tile_clip_polygon(tile_bounds, extent);

        if tile_clip_poly.is_none() {
            return Ok(None);
        }

        let tile_clip_poly = tile_clip_poly.unwrap();

        let mut clipped_layer = Layer {
            version: layer.version,
            name: layer.name.clone(),
            features: vec![],
            keys: layer.keys.clone(),
            values: layer.values.clone(),
            extent: layer.extent,
        };

        for feature in &layer.features {
            if let Some(clipped_feature) =
                self.clip_feature(feature, &tile_clip_poly, extent, tile_bounds)?
            {
                clipped_layer.features.push(clipped_feature);
            }
        }

        if clipped_layer.features.is_empty() {
            return Ok(None);
        }

        Ok(Some(clipped_layer))
    }

    fn clip_feature(
        &self,
        feature: &vector_tile::tile::Feature,
        tile_clip_poly: &MultiPolygon,
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> Result<Option<vector_tile::tile::Feature>> {
        let geom_type = feature.get_type();

        match geom_type {
            GeomType::Point => {
                self.clip_point_feature(feature, tile_clip_poly, extent, tile_bounds)
            }
            GeomType::Linestring => {
                self.clip_linestring_feature(feature, tile_clip_poly, extent, tile_bounds)
            }
            GeomType::Polygon => {
                self.clip_polygon_feature(feature, tile_clip_poly, extent, tile_bounds)
            }
            _ => Ok(Some(feature.clone())),
        }
    }

    fn clip_point_feature(
        &self,
        feature: &vector_tile::tile::Feature,
        tile_clip_poly: &MultiPolygon,
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> Result<Option<vector_tile::tile::Feature>> {
        let points = Self::decode_geometry(&feature.geometry, GeomType::Point);
        let mut clipped_points = Vec::new();

        for (cmd, coords) in points {
            let mut clipped_coords = Vec::new();
            for coord in coords {
                let geo_coord = Self::tile_coord_to_geo(coord, extent, tile_bounds);
                let point: geo::Point = geo_coord.into();

                if tile_clip_poly.iter().any(|poly| poly.contains(&point)) {
                    clipped_coords.push(coord);
                }
            }

            if !clipped_coords.is_empty() {
                clipped_points.push((cmd, clipped_coords));
            }
        }

        if clipped_points.is_empty() {
            return Ok(None);
        }

        let encoded_geom = Self::encode_geometry(&clipped_points);
        Ok(Some(vector_tile::tile::Feature {
            id: feature.id,
            tags: feature.tags.clone(),
            geom_type: feature.geom_type,
            geometry: encoded_geom,
        }))
    }

    fn clip_linestring_feature(
        &self,
        feature: &vector_tile::tile::Feature,
        tile_clip_poly: &MultiPolygon,
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> Result<Option<vector_tile::tile::Feature>> {
        let lines = Self::decode_geometry(&feature.geometry, GeomType::Linestring);
        let mut clipped_lines = Vec::new();

        for (_cmd, coords) in lines {
            if coords.len() < 2 {
                continue;
            }

            let geo_coords: Vec<Coord> = coords
                .iter()
                .map(|&c| Self::tile_coord_to_geo(c, extent, tile_bounds))
                .collect();

            let linestring = LineString::from(geo_coords);

            // Simple intersection check - keep linestring if it intersects the clip polygon
            // TODO: Implement true geometric line clipping for more precise results
            let mut intersects_clip = false;
            for poly in tile_clip_poly.iter() {
                if linestring.intersects(poly) {
                    intersects_clip = true;
                    break;
                }
            }

            if intersects_clip {
                let tile_coords: Vec<(i32, i32)> = linestring
                    .coords()
                    .map(|c| Self::geo_coord_to_tile(c, extent, tile_bounds))
                    .collect();

                if !tile_coords.is_empty() {
                    clipped_lines.push((2, tile_coords));
                }
            }
        }

        if clipped_lines.is_empty() {
            return Ok(None);
        }

        let encoded_geom = Self::encode_geometry(&clipped_lines);
        Ok(Some(vector_tile::tile::Feature {
            id: feature.id,
            tags: feature.tags.clone(),
            geom_type: feature.geom_type,
            geometry: encoded_geom,
        }))
    }

    fn clip_polygon_feature(
        &self,
        feature: &vector_tile::tile::Feature,
        tile_clip_poly: &MultiPolygon,
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> Result<Option<vector_tile::tile::Feature>> {
        let rings = Self::decode_geometry(&feature.geometry, GeomType::Polygon);
        let mut current_exterior: Vec<Coord> = Vec::new();
        let mut current_holes: Vec<LineString> = Vec::new();
        let mut polygons: Vec<Polygon> = Vec::new();

        for (cmd, coords) in rings {
            let geo_coords: Vec<Coord> = coords
                .iter()
                .map(|&c| Self::tile_coord_to_geo(c, extent, tile_bounds))
                .collect();

            if cmd == 1 || current_exterior.is_empty() {
                if !current_exterior.is_empty() {
                    let poly = Polygon::new(
                        LineString::from(current_exterior.clone()),
                        current_holes.drain(..).collect(),
                    );
                    polygons.push(poly);
                }
                current_exterior = geo_coords;
            } else {
                current_holes.push(LineString::from(geo_coords));
            }
        }

        if !current_exterior.is_empty() {
            let poly = Polygon::new(LineString::from(current_exterior), current_holes);
            polygons.push(poly);
        }

        let feature_multipoly = MultiPolygon::from(polygons);

        let clipped = feature_multipoly.intersection(tile_clip_poly);

        if clipped.0.is_empty() {
            return Ok(None);
        }

        let mut encoded_rings = Vec::new();

        for poly in &clipped.0 {
            let exterior_coords: Vec<(i32, i32)> = poly
                .exterior()
                .coords()
                .map(|c| Self::geo_coord_to_tile(c, extent, tile_bounds))
                .collect();

            if exterior_coords.len() >= 3 {
                encoded_rings.push((1, exterior_coords));
            }

            for hole in poly.interiors() {
                let hole_coords: Vec<(i32, i32)> = hole
                    .coords()
                    .map(|c| Self::geo_coord_to_tile(c, extent, tile_bounds))
                    .collect();

                if hole_coords.len() >= 3 {
                    encoded_rings.push((2, hole_coords));
                }
            }
        }

        if encoded_rings.is_empty() {
            return Ok(None);
        }

        let encoded_geom = Self::encode_geometry(&encoded_rings);
        Ok(Some(vector_tile::tile::Feature {
            id: feature.id,
            tags: feature.tags.clone(),
            geom_type: feature.geom_type,
            geometry: encoded_geom,
        }))
    }

    fn extract_linestrings_from_geometry(geom: &geo::Geometry) -> Vec<LineString> {
        match geom {
            geo::Geometry::LineString(ls) => vec![ls.clone()],
            geo::Geometry::MultiLineString(mls) => mls.0.clone(),
            geo::Geometry::GeometryCollection(gc) => {
                gc.0.iter()
                    .flat_map(Self::extract_linestrings_from_geometry)
                    .collect()
            }
            _ => vec![],
        }
    }

    fn extract_polygons_from_geometry(geom: &geo::Geometry) -> Vec<Polygon> {
        match geom {
            geo::Geometry::Polygon(p) => vec![p.clone()],
            geo::Geometry::MultiPolygon(mp) => mp.0.clone(),
            geo::Geometry::GeometryCollection(gc) => {
                gc.0.iter()
                    .flat_map(Self::extract_polygons_from_geometry)
                    .collect()
            }
            _ => vec![],
        }
    }

    fn get_tile_clip_polygon(
        &self,
        tile_bounds: (f64, f64, f64, f64),
        _extent: u32,
    ) -> Option<MultiPolygon> {
        let (min_lon, min_lat, max_lon, max_lat) = tile_bounds;

        let tile_poly = Polygon::new(
            LineString::from(vec![
                Coord {
                    x: min_lon,
                    y: min_lat,
                },
                Coord {
                    x: max_lon,
                    y: min_lat,
                },
                Coord {
                    x: max_lon,
                    y: max_lat,
                },
                Coord {
                    x: min_lon,
                    y: max_lat,
                },
                Coord {
                    x: min_lon,
                    y: min_lat,
                },
            ]),
            vec![],
        );

        let tile_multipoly = MultiPolygon::from(vec![tile_poly]);
        let clipped = self.clip_polygon.intersection(&tile_multipoly);

        if clipped.0.is_empty() {
            None
        } else {
            Some(clipped)
        }
    }

    fn tile_to_geo_bounds(z: u8, x: u32, y: u32) -> (f64, f64, f64, f64) {
        let n = 2_f64.powi(z as i32);
        let lon_min = (x as f64) / n * 360.0 - 180.0;
        let lon_max = ((x + 1) as f64) / n * 360.0 - 180.0;

        let lat_min = Self::tile_y_to_lat(y + 1, z);
        let lat_max = Self::tile_y_to_lat(y, z);

        (lon_min, lat_min, lon_max, lat_max)
    }

    fn tile_y_to_lat(y: u32, z: u8) -> f64 {
        let n = 2_f64.powi(z as i32);
        let lat_rad = ((std::f64::consts::PI * (1.0 - 2.0 * (y as f64) / n)).sinh()).atan();
        lat_rad.to_degrees()
    }

    fn tile_coord_to_geo(
        coord: (i32, i32),
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> Coord {
        let (min_lon, min_lat, max_lon, max_lat) = tile_bounds;
        let x_frac = coord.0 as f64 / extent as f64;
        let y_frac = coord.1 as f64 / extent as f64;

        Coord {
            x: min_lon + (max_lon - min_lon) * x_frac,
            y: max_lat - (max_lat - min_lat) * y_frac,
        }
    }

    fn geo_coord_to_tile(
        coord: &Coord,
        extent: u32,
        tile_bounds: (f64, f64, f64, f64),
    ) -> (i32, i32) {
        let (min_lon, min_lat, max_lon, max_lat) = tile_bounds;

        let x_frac = (coord.x - min_lon) / (max_lon - min_lon);
        let y_frac = (max_lat - coord.y) / (max_lat - min_lat);

        let x = (x_frac * extent as f64).round() as i32;
        let y = (y_frac * extent as f64).round() as i32;

        (x.clamp(0, extent as i32), y.clamp(0, extent as i32))
    }

    fn decode_geometry(geometry: &[u32], _geom_type: GeomType) -> Vec<(u32, Vec<(i32, i32)>)> {
        let mut result = Vec::new();
        let mut i = 0;
        let mut x = 0i32;
        let mut y = 0i32;

        while i < geometry.len() {
            let cmd_int = geometry[i];
            let cmd = cmd_int & 0x7;
            let count = cmd_int >> 3;
            i += 1;

            let mut coords = Vec::new();

            match cmd {
                1 | 2 => {
                    for _ in 0..count {
                        if i + 1 >= geometry.len() {
                            break;
                        }

                        let dx = Self::zigzag_decode(geometry[i]);
                        let dy = Self::zigzag_decode(geometry[i + 1]);
                        i += 2;

                        x += dx;
                        y += dy;

                        coords.push((x, y));
                    }
                    result.push((cmd, coords));
                }
                7 => {
                    result.push((cmd, vec![]));
                }
                _ => {}
            }
        }

        result
    }

    fn encode_geometry(commands: &[(u32, Vec<(i32, i32)>)]) -> Vec<u32> {
        let mut result = Vec::new();
        let mut x = 0i32;
        let mut y = 0i32;

        for (cmd, coords) in commands {
            if *cmd == 7 {
                result.push((*cmd & 0x7) | (1 << 3));
                continue;
            }

            let count = coords.len() as u32;
            result.push((*cmd & 0x7) | (count << 3));

            for &(cx, cy) in coords {
                let dx = cx - x;
                let dy = cy - y;

                result.push(Self::zigzag_encode(dx));
                result.push(Self::zigzag_encode(dy));

                x = cx;
                y = cy;
            }
        }

        result
    }

    fn zigzag_decode(n: u32) -> i32 {
        ((n >> 1) as i32) ^ (-((n & 1) as i32))
    }

    fn zigzag_encode(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }
}

fn load_geojson_multipolygon(path: &PathBuf) -> Result<MultiPolygon> {
    let geojson_str = fs::read_to_string(path)
        .with_context(|| format!("Failed to read GeoJSON file: {}", path.display()))?;

    let geojson = geojson_str
        .parse::<GeoJson>()
        .context("Failed to parse GeoJSON")?;

    let geometry = match geojson {
        GeoJson::Geometry(geom) => geom,
        GeoJson::Feature(feature) => feature.geometry.context("Feature has no geometry")?,
        GeoJson::FeatureCollection(fc) => fc
            .features
            .into_iter()
            .find_map(|f| f.geometry)
            .context("No features with geometry found")?,
    };

    match geometry.value {
        Value::Polygon(coords) => {
            let exterior: Vec<Coord> = coords[0]
                .iter()
                .map(|c| Coord { x: c[0], y: c[1] })
                .collect();
            let holes: Vec<LineString> = coords[1..]
                .iter()
                .map(|ring| {
                    let coords: Vec<Coord> =
                        ring.iter().map(|c| Coord { x: c[0], y: c[1] }).collect();
                    LineString::from(coords)
                })
                .collect();

            Ok(MultiPolygon::from(vec![Polygon::new(
                LineString::from(exterior),
                holes,
            )]))
        }
        Value::MultiPolygon(coords) => {
            let polygons: Vec<Polygon> = coords
                .iter()
                .map(|poly_coords| {
                    let exterior: Vec<Coord> = poly_coords[0]
                        .iter()
                        .map(|c| Coord { x: c[0], y: c[1] })
                        .collect();
                    let holes: Vec<LineString> = poly_coords[1..]
                        .iter()
                        .map(|ring| {
                            let coords: Vec<Coord> =
                                ring.iter().map(|c| Coord { x: c[0], y: c[1] }).collect();
                            LineString::from(coords)
                        })
                        .collect();

                    Polygon::new(LineString::from(exterior), holes)
                })
                .collect();

            Ok(MultiPolygon::from(polygons))
        }
        _ => anyhow::bail!("GeoJSON must contain a Polygon or MultiPolygon"),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("=== Vector Tile Clipping Tool ===\n");

    // Load clip polygon
    println!("Loading GeoJSON boundary from: {}", args.geojson.display());
    let clip_polygon = load_geojson_multipolygon(&args.geojson)?;
    println!(
        "✓ Loaded clip polygon with {} polygon(s)\n",
        clip_polygon.0.len()
    );

    // Create clipper
    let clipper = TileClipper::new(clip_polygon, args.extent);

    // Create output directory
    fs::create_dir_all(&args.output)?;

    // Process tile directory
    process_directory(&args, &clipper)?;

    Ok(())
}

fn process_directory(args: &Args, clipper: &TileClipper) -> Result<()> {
    println!("Processing tiles from directory: {}", args.input.display());
    let mut tiles_processed = 0;
    let mut tiles_kept = 0;

    // Walk through input directory to find all tiles
    for entry in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Parse z/x/y from path (expecting structure like z/x/y.mvt or z/x/y.pbf)
        if let Some((z, x, y)) = parse_tile_path(path) {
            tiles_processed += 1;

            // Read tile
            let mut file = File::open(path)?;
            let mut tile_data = Vec::new();
            file.read_to_end(&mut tile_data)?;

            // Decompress if needed
            let decompressed = if args.gzip {
                decompress_tile(&tile_data)?
            } else {
                tile_data
            };

            // Clip tile
            match clipper.clip_tile(&decompressed, z, x, y) {
                Ok(Some(clipped_data)) => {
                    // Recompress if needed
                    let output_data = if args.gzip {
                        compress_tile(&clipped_data)?
                    } else {
                        clipped_data
                    };

                    // Write output tile
                    let output_dir = args.output.join(format!("{}", z)).join(format!("{}", x));
                    fs::create_dir_all(&output_dir)?;

                    let output_path = output_dir.join(format!("{}.mvt", y));
                    let mut output_file = File::create(&output_path)?;
                    output_file.write_all(&output_data)?;

                    tiles_kept += 1;

                    if tiles_processed % 100 == 0 {
                        println!(
                            "  Processed {} tiles, kept {} tiles",
                            tiles_processed, tiles_kept
                        );
                    }
                }
                Ok(None) => {
                    // Tile was completely clipped out
                }
                Err(e) => {
                    eprintln!("  Warning: Failed to clip tile {}/{}/{}: {}", z, x, y, e);
                }
            }
        }
    }

    println!(
        "\n✓ Processing complete: {} tiles processed, {} tiles kept",
        tiles_processed, tiles_kept
    );
    println!("✓ Output written to: {}", args.output.display());
    println!("\nDone!");

    Ok(())
}

fn parse_tile_path(path: &std::path::Path) -> Option<(u8, u32, u32)> {
    // Extract z/x/y from path like "tiles/14/8192/5461.mvt"
    let components: Vec<_> = path.components().rev().take(3).collect();
    if components.len() < 3 {
        return None;
    }

    let y_str = components[0].as_os_str().to_str()?.split('.').next()?;
    let x_str = components[1].as_os_str().to_str()?;
    let z_str = components[2].as_os_str().to_str()?;

    let y: u32 = y_str.parse().ok()?;
    let x: u32 = x_str.parse().ok()?;
    let z: u8 = z_str.parse().ok()?;

    Some((z, x, y))
}

fn decompress_tile(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress gzip tile")?;
    Ok(decompressed)
}

fn compress_tile(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    encoder.finish().context("Failed to compress tile")
}
