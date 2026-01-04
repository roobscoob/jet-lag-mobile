use jet_lag_core::shape::Shape;

use crate::render::style::Style;

pub mod style;
mod thread;

pub struct RenderHandle {}

pub struct RenderSession {}

impl RenderSession {
    pub fn append_shape(&mut self, shape: &dyn Shape, style: Style) -> RenderHandle {
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
