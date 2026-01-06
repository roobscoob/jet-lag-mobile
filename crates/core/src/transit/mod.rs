use chrono::NaiveDate;
use geo::{LineString, Point};
use std::sync::Arc;

// ============================================================================
// Identifiers
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StationIdentifier(Arc<str>);

impl StationIdentifier {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteIdentifier(Arc<str>);

impl RouteIdentifier {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TripIdentifier(Arc<str>);

impl TripIdentifier {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexIdentifier(Arc<str>);

impl ComplexIdentifier {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(s.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Enums
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteType {
    Tram,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLift,
    Funicular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirectionId {
    Outbound = 0,
    Inbound = 1,
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Clone, Debug)]
pub struct StopEvent {
    pub station_id: StationIdentifier,
    pub arrival: u32, // seconds since service day start
    pub departure: u32,
    pub stop_sequence: u32,
}

// ============================================================================
// Traits
// ============================================================================

/// A single vehicle run with specific stops and times
pub trait Trip: Send + Sync {
    fn id(&self) -> TripIdentifier;
    fn route_id(&self) -> RouteIdentifier;

    /// Ordered stop events for this trip
    fn stop_events(&self) -> &[StopEvent];

    /// Which days does this trip run?
    fn runs_on(&self, date: NaiveDate) -> bool;

    /// Direction (e.g., Northbound vs Southbound)
    fn direction_id(&self) -> DirectionId;

    /// Display name (e.g., "Downtown", "To City Center")
    fn headsign(&self) -> &str;
}

/// A transit route (e.g., "Red Line", "Route 66")
pub trait Route: Send + Sync {
    fn id(&self) -> RouteIdentifier;

    /// Type of transportation
    fn route_type(&self) -> RouteType;

    /// Short name (e.g., "1", "A", "Red")
    fn short_name(&self) -> &str;

    /// Long name (e.g., "Broadway-7th Ave Local")
    fn long_name(&self) -> &str;

    /// Physical path the route takes (for spatial queries)
    fn geometry(&self) -> &LineString;

    /// All trips on this route
    fn trips(&self) -> &[Arc<dyn Trip>];
}

/// A transit station (single boarding location)
pub trait TransitStation: Send + Sync {
    fn identifier(&self) -> StationIdentifier;
    fn name(&self) -> &str;
    fn location(&self) -> Point;

    fn complex(&self) -> Arc<dyn TransitComplex>;
}

/// A complex of connected stations (e.g., Times Square, Union Station)
pub trait TransitComplex: Send + Sync {
    fn identifier(&self) -> ComplexIdentifier;
    fn name(&self) -> &str;

    fn all_stations(&self) -> &[Arc<dyn TransitStation>];

    /// Approximate center point for the complex
    fn center(&self) -> Point;
}

/// Provider of all transit data with lookup methods
pub trait TransitProvider: Send + Sync {
    // ---- Collections ----
    fn all_stations(&self) -> &[Arc<dyn TransitStation>];
    fn all_complexes(&self) -> &[Arc<dyn TransitComplex>];
    fn all_routes(&self) -> &[Arc<dyn Route>];

    // ---- Lookups ----
    fn get_station(&self, id: &StationIdentifier) -> Option<Arc<dyn TransitStation>>;
    fn get_complex(&self, id: &ComplexIdentifier) -> Option<Arc<dyn TransitComplex>>;
    fn get_route(&self, id: &RouteIdentifier) -> Option<Arc<dyn Route>>;
    fn get_trip(&self, id: &TripIdentifier) -> Option<Arc<dyn Trip>>;

    // ---- Spatial queries (implement with R-tree) ----

    /// Find stations within radius (meters)
    fn stations_near(&self, point: Point, radius_m: f64) -> Vec<Arc<dyn TransitStation>>;

    /// Find routes within radius (meters)
    fn routes_near(&self, point: Point, radius_m: f64) -> Vec<Arc<dyn Route>>;
}
