use std::sync::Arc;

use crate::shape::{
    compiler::Register,
    types::{Centimeters, Position},
};

pub enum SdfInstruction {
    Point {
        position: Position,
        output: Register,
    },
    Union {
        shapes: Vec<Register>,
        output: Register,
    },
    Intersection {
        left: Register,
        right: Register,
        output: Register,
    },
    Subtract {
        left: Register,
        right: Register,
        output: Register,
    },
    Invert {
        input: Register,
        output: Register,
    },
    Dilate {
        input: Register,
        amount: Centimeters,
        output: Register,
    },
    Boundary {
        inside: Register,
        outside: Register,
        output: Register,
    },
    LoadVdg {
        diagram: Arc<boostvoronoi::prelude::Diagram>,
        output: Register,
    },
}
