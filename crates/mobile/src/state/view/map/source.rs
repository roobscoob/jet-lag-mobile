use std::ffi::OsString;

pub struct MapSource {
    pub(crate) pmtiles_path: OsString,
    pub(crate) bounds_path: OsString,
}

impl MapSource {
    pub(crate) fn nyc() -> Self {
        MapSource {
            pmtiles_path: "/data/data/ly.hall.jetlagmobile/files/nyc_tiles.pmtiles".into(),
            bounds_path: "/data/data/ly.hall.jetlagmobile/files/nyc_bounds.geojson".into(),
        }
    }
}
