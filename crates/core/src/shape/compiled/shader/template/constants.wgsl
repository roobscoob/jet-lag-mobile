#define_import_path template::constants

override USE_ELLIPSOID: bool = true;
override USE_HIGH_PRECISION: bool = false;

const COORD_SCALE: i32 = 10000000; // lat/lon stored as degrees * 1e7
const WGS84_A: f32 = 6378137.0;
const WGS84_B: f32 = 6356752.314245;
const WGS84_F: f32 = 1.0 / 298.257223563;
const EARTH_RADIUS: f32 = 6371000.0;
const DEG_TO_RAD: f32 = 0.017453292519943295;

