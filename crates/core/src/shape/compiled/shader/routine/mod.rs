pub mod dilate;
pub mod edge;
pub mod intersection;
pub mod invert;
pub mod point;
pub mod point_cloud;
pub mod subtract;
pub mod union;

pub struct RoutineResult {
    pub argument_len: u8,
    pub variable: naga::Handle<naga::LocalVariable>,
}
