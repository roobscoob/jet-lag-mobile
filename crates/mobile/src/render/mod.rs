use actix::Addr;
use geo::Point;
use jet_lag_core::shape::{
    Shape,
    compiler::{Register, SdfCompiler},
};

use crate::render::{
    style::Style,
    thread::{RenderThread, start_render_thread},
};

pub mod style;
pub mod thread;

pub struct RenderHandle {
    register: Register,
    style: Style,
}

pub struct RenderSession {
    pub render_thread: Addr<RenderThread>,
    compiler: SdfCompiler,
}

impl RenderSession {
    pub fn new() -> Self {
        let render_thread = start_render_thread();
        let compiler = SdfCompiler::new();
        Self {
            render_thread,
            compiler,
        }
    }

    pub fn test(&mut self) -> Register {
        let register = self.compiler.point(Point::new(0.0, 0.0));

        register
    }

    pub fn append_shape(&mut self, shape: &dyn Shape, style: Style) -> RenderHandle {
        let register = shape.build_into(&mut self.compiler);

        todo!()
    }
}

impl RenderHandle {
    pub fn update_style(&mut self, style: Style) {
        todo!()
    }

    pub fn remove(self) {
        todo!()
    }
}
