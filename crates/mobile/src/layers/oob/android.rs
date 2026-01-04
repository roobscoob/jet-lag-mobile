use crate::layers::android::CustomLayer;
use glow::HasContext;
use spatialtree::{QuadTree, QuadVec};

enum TileEntry {
  Loaded {
    texture: glow::Renderbuffer
  }
}

pub struct OutOfBoundsLayer {
  active_tile_requests: Vec<QuadTree<TileEntry, QuadVec>>
}

impl CustomLayer for OutOfBoundsLayer {
  fn new() -> eyre::Result<Self> {
    Ok(Self {
      active_tile_requests: vec![QuadTree::new()]
    })
  }

  fn render(&mut self, parameters: &crate::layers::android::Parameters) -> eyre::Result<()> {
    let zoom = parameters.zoom.max(0.0) as usize;
    let zoom_layer = self.active_tile_requests.get(zoom).or_else(|| self.active_tile_requests.last());
    
    Ok(())
  }

  fn context_lost(&mut self) {}

  fn cleanup(self) {}
}
