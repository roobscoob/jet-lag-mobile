// Pre-generated vector tile protobuf module
// Based on https://github.com/mapbox/vector-tile-spec

use prost::Message;

pub mod tile {
    use prost::Message;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
    #[repr(i32)]
    pub enum GeomType {
        #[default]
        Unknown = 0,
        Point = 1,
        Linestring = 2,
        Polygon = 3,
    }

    impl GeomType {
        pub fn as_str_name(&self) -> &'static str {
            match self {
                GeomType::Unknown => "UNKNOWN",
                GeomType::Point => "POINT",
                GeomType::Linestring => "LINESTRING",
                GeomType::Polygon => "POLYGON",
            }
        }
    }

    impl TryFrom<i32> for GeomType {
        type Error = i32;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(GeomType::Unknown),
                1 => Ok(GeomType::Point),
                2 => Ok(GeomType::Linestring),
                3 => Ok(GeomType::Polygon),
                _ => Err(value),
            }
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Value {
        #[prost(string, optional, tag = "1")]
        pub string_value: Option<String>,
        #[prost(float, optional, tag = "2")]
        pub float_value: Option<f32>,
        #[prost(double, optional, tag = "3")]
        pub double_value: Option<f64>,
        #[prost(int64, optional, tag = "4")]
        pub int_value: Option<i64>,
        #[prost(uint64, optional, tag = "5")]
        pub uint_value: Option<u64>,
        #[prost(sint64, optional, tag = "6")]
        pub sint_value: Option<i64>,
        #[prost(bool, optional, tag = "7")]
        pub bool_value: Option<bool>,
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Feature {
        #[prost(uint64, optional, tag = "1")]
        pub id: Option<u64>,
        #[prost(uint32, repeated, packed = "true", tag = "2")]
        pub tags: Vec<u32>,
        #[prost(enumeration = "GeomType", optional, tag = "3")]
        pub geom_type: Option<i32>,
        #[prost(uint32, repeated, packed = "true", tag = "4")]
        pub geometry: Vec<u32>,
    }

    impl Feature {
        pub fn get_type(&self) -> GeomType {
            match self.geom_type {
                Some(1) => GeomType::Point,
                Some(2) => GeomType::Linestring,
                Some(3) => GeomType::Polygon,
                _ => GeomType::Unknown,
            }
        }
    }

    #[derive(Clone, PartialEq, Message)]
    pub struct Layer {
        #[prost(uint32, required, tag = "15", default = "1")]
        pub version: u32,
        #[prost(string, required, tag = "1")]
        pub name: String,
        #[prost(message, repeated, tag = "2")]
        pub features: Vec<Feature>,
        #[prost(string, repeated, tag = "3")]
        pub keys: Vec<String>,
        #[prost(message, repeated, tag = "4")]
        pub values: Vec<Value>,
        #[prost(uint32, optional, tag = "5", default = "4096")]
        pub extent: Option<u32>,
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Tile {
    #[prost(message, repeated, tag = "3")]
    pub layers: Vec<tile::Layer>,
}
