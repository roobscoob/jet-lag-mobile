use crate::layers::android::{CustomLayer, gl::get_gl_context};
use glow::HasContext;

pub struct OutOfBoundsLayer {}

impl CustomLayer for OutOfBoundsLayer {
  fn new() -> eyre::Result<Self> {
    Ok(Self {})
  }

  fn render(&mut self, parameters: &crate::layers::android::Parameters) -> eyre::Result<()> {
    Ok(())
  }

  fn context_lost(&mut self) {
  }

  fn cleanup(self) {
    
  }
}
