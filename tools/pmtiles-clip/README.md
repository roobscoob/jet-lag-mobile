# PMTiles/Vector Tile Clipping Tool

A Rust tool for **truly clipping vector tiles** using a GeoJSON boundary. Unlike simple tile filtering, this tool performs geometric clipping operations on individual features, cutting them at the exact boundary.

## Features

- **True Geometric Clipping**: Clips points, linestrings, and polygons at the exact boundary
- **Preserves Attributes**: All feature properties are maintained
- **Multiple Geometry Types**: Handles points, linestrings, and polygons
- **Efficient Processing**: Only processes tiles that exist
- **GeoJSON Support**: Accepts Polygon or MultiPolygon boundaries
- **Compression Support**: Handles gzip-compressed tiles

## Installation

```bash
cd tools/pmtiles-clip
cargo build --release
```

The binary will be at `target/release/pmtiles-clip` (or `pmtiles-clip.exe` on Windows).

## Usage

### Basic Usage

```bash
pmtiles-clip \
  --input tiles/ \
  --output clipped-tiles/ \
  --geojson boundary.geojson
```

### Arguments

- `-i, --input <DIR>`: Input tile directory (z/x/y.mvt structure)
- `-o, --output <DIR>`: Output tile directory
- `-g, --geojson <FILE>`: GeoJSON file containing boundary (Polygon or MultiPolygon)
- `-e, --extent <NUMBER>`: Tile extent in pixels (default: 4096)
- `--gzip`: Input tiles are gzip compressed

### Working with PMTiles

To use with PMTiles files, you'll need the `pmtiles` CLI tool:

1. **Extract tiles from PMTiles**:
   ```bash
   pmtiles extract input.pmtiles tiles/
   ```

2. **Clip the tiles**:
   ```bash
   pmtiles-clip -i tiles/ -o clipped-tiles/ -g boundary.geojson
   ```

3. **Create new PMTiles from clipped tiles**:
   ```bash
   pmtiles convert clipped-tiles/ output.pmtiles
   ```

Install the `pmtiles` CLI from: https://github.com/protomaps/go-pmtiles

## How It Works

1. **Loads GeoJSON Boundary**: Reads a Polygon or MultiPolygon from a GeoJSON file
2. **Processes Tiles**: Walks through the input tile directory
3. **Decodes Vector Tiles**: Parses Mapbox Vector Tile (MVT) protobuf format
4. **Performs Geometric Clipping**:
   - For **points**: Tests if they fall within the boundary
   - For **linestrings**: Checks intersection with the boundary polygon
   - For **polygons**: Computes geometric intersection, preserving holes
5. **Re-encodes Tiles**: Encodes clipped features back to MVT format
6. **Outputs Results**: Writes clipped tiles to output directory

## GeoJSON Format

The GeoJSON file should contain either:
- A single `Polygon` geometry
- A single `MultiPolygon` geometry
- A `Feature` with one of the above geometries
- A `FeatureCollection` where at least one feature has a geometry

Example GeoJSON:
```json
{
  "type": "Feature",
  "geometry": {
    "type": "Polygon",
    "coordinates": [[
      [-122.5, 37.7],
      [-122.5, 37.8],
      [-122.4, 37.8],
      [-122.4, 37.7],
      [-122.5, 37.7]
    ]]
  }
}
```

## Input Tile Directory Structure

Tiles should be organized in z/x/y structure:
```
tiles/
├── 14/
│   ├── 8192/
│   │   ├── 5461.mvt
│   │   ├── 5462.mvt
│   │   └── ...
│   └── 8193/
│       └── ...
├── 15/
│   └── ...
```

## Implementation Details

### Coordinate Transformation

The tool converts between:
- **Tile coordinates**: Integer coordinates in the tile's local coordinate system (0 to extent)
- **Geographic coordinates**: Lat/lon in WGS84
- **Web Mercator tiles**: Standard XYZ tile addressing

### Clipping Algorithm

1. For each tile, compute the intersection between the tile bounds and the clip polygon
2. For each feature in the tile:
   - Decode the MVT geometry into geographic coordinates
   - Perform geometric operations:
     - Points: Exact point-in-polygon test
     - Linestrings: Intersection check (keeps line if it intersects)
     - Polygons: True geometric intersection using the `geo` crate's BooleanOps
   - Convert clipped geometry back to tile coordinates
   - Re-encode to MVT format

### Geometry Encoding

The tool handles Mapbox Vector Tile encoding:
- Command-based geometry encoding (MoveTo, LineTo, ClosePath)
- Zigzag encoding for delta-encoded coordinates
- Proper handling of polygon rings (exterior vs holes)

## Example

See `example-boundary.geojson` for a sample boundary file.

## Dependencies

- `geo` + `geo-types`: Geometric operations (intersection, clipping)
- `geojson`: GeoJSON parsing
- `prost`: Protocol buffer encoding/decoding for MVT
- `anyhow`: Error handling
- `clap`: CLI argument parsing
- `flate2`: Gzip compression
- `walkdir`: Directory traversal

## Future Enhancements

- [ ] True geometric line clipping (currently only checks intersection)
- [ ] Parallel tile processing for better performance
- [ ] Progress bar for long-running operations
- [ ] Support for attribute filtering
- [ ] Buffer zone support (clip with a boundary buffer)

## License

MIT
