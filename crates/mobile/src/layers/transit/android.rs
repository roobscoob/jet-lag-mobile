use std::{cell::Cell, f64::consts::PI, sync::LazyLock};

use eyre::{ContextCompat, bail};
use glam::{Vec2, dvec3, dvec4, vec3, vec4};
use glow::{HasContext, NativeBuffer, NativeProgram, NativeUniformLocation};
use khronos_egl::{DynamicInstance, EGL1_0};
use mercantile::{LngLat, convert_xy};
use tracing::{debug, error, info};
struct SimpleGraphics {
  pos_attrib: u32,
  proj_uniform: NativeUniformLocation,
  fill_color_uniform: NativeUniformLocation,
  buffer: NativeBuffer,
  program: NativeProgram,
  debug_counter: Cell<u16>,
}

const SQUARE_OFFSET: Vec2 = Vec2::new(40.7571418, -73.9805655);

impl SimpleGraphics {
  fn new(gl: &glow::Context, program: NativeProgram) -> eyre::Result<Self> {
    use glow::*;
    unsafe {
      let pos_attrib = gl
        .get_attrib_location(program, "a_pos")
        .context("no a_pos attribute")?;
      let fill_color_uniform = gl
        .get_uniform_location(program, "fill_color")
        .context("no fill_color uniform")?;
      let proj_uniform = gl
        .get_uniform_location(program, "proj")
        .context("no proj uniform")?;

      static BACKGROUND: [f32; 8] = {
        let mut base = [-5.0, -5.0, 5.0, -5.0, -5.0, 5.0, 5.0, 5.0];
        base
      };

      let buffer = gl.create_buffer().wrap_gl()?;
      gl.bind_buffer(ARRAY_BUFFER, Some(buffer));
      gl.buffer_data_u8_slice(ARRAY_BUFFER, BACKGROUND.as_bytes(), STATIC_DRAW);

      Ok(Self {
        pos_attrib,
        proj_uniform,
        fill_color_uniform,
        // zoom_level_uniform,
        buffer,
        program,
        debug_counter: Cell::new(0),
      })
    }
  }

  fn render(&self, gl: &glow::Context, parameters: &Parameters) -> eyre::Result<()> {
    use glow::*;
    unsafe {
      gl.use_program(Some(self.program));
      gl.bind_buffer(ARRAY_BUFFER, Some(self.buffer));
      gl.enable_vertex_attrib_array(self.pos_attrib);
      gl.vertex_attrib_pointer_f32(self.pos_attrib, 2, FLOAT, false, 0, 0);
      gl.disable(STENCIL_TEST);
      gl.disable(DEPTH_TEST);
      gl.uniform_4_f32(
        Some(&self.fill_color_uniform),
        100.0 / 255.0,
        149.0 / 255.0,
        237.0 / 255.0,
        1.0,
      );
      let mat = glam::DMat4::from_cols_slice(&parameters.projection_matrix);
      let lng_lat = LngLat {
        lng: SQUARE_OFFSET.y as f64,
        lat: SQUARE_OFFSET.x as f64,
      };
      let mercator_x = (lng_lat.lng + 180.0) / 360.0;
      let lat_rad = lng_lat.lat * PI / 180.0;
      let mercator_y = (1.0 - f64::ln(f64::tan(lat_rad) + 1.0 / f64::cos(lat_rad)) / PI) / 2.0;
      let zoom = 2.0f64.powf(parameters.zoom);
      let extent = 512.0;
      let mercator_offset = dvec3(zoom * extent * mercator_x, zoom * extent * mercator_y, 0.0);
      let mat = mat.mul_mat4(&glam::DMat4::from_translation(mercator_offset));

      let pos = mat.mul_vec4(dvec4(0.0, 0.0, 0.0, 1.0));
      if self.debug_counter.get() == 0 {
        debug!("render pos 5,5: {pos},,,, {zoom},,,, {mercator_offset},,,, {parameters:#?}");
        self.debug_counter.set(50);
      } else {
        self.debug_counter.update(|x| x - 1);
      }
      let mat = mat.to_cols_array().map(|v| v as f32);
      gl.uniform_matrix_4_f32_slice(Some(&self.proj_uniform), false, &mat);
      gl.draw_arrays(TRIANGLE_STRIP, 0, 4);
    }
    Ok(())
  }

  fn cleanup(self, gl: &glow::Context) {}
}

struct TransitLayer {
  gl: glow::Context,
  program: Option<NativeProgram>,
  graphics: Option<SimpleGraphics>,
}

impl TransitLayer {
  fn new() -> Self {
    tracing::info!("setting up context");
    static DYNAMIC: LazyLock<DynamicInstance<EGL1_0>> =
      LazyLock::new(|| unsafe { DynamicInstance::load().expect("failed to obtain egl instance") });

    let gl = unsafe {
      glow::Context::from_loader_function(move |str| {
        DYNAMIC
          .get_proc_address(str)
          .map(|x| x as *const _)
          .unwrap_or_default()
      })
    };

    info!("got gl context!");
    let program = Self::create_program(&gl).expect("failed to setup shader program");

    info!("prepared shader program");

    let graphics = SimpleGraphics::new(&gl, program).expect("failed to setup graphics");

    info!("graphics are up!");

    Self {
      gl,
      program: Some(program),
      graphics: Some(graphics),
    }
  }

  fn create_program(gl: &glow::Context) -> eyre::Result<NativeProgram> {
    use glow::*;
    unsafe {
      let check_compile_status = |shader: NativeShader, kind: &str| -> eyre::Result<()> {
        if !gl.get_shader_compile_status(shader) {
          bail!("[{kind}]: {}", gl.get_shader_info_log(shader));
        }
        Ok(())
      };
      let program = gl.create_program().wrap_gl()?;
      let vertex_shader = gl.create_shader(VERTEX_SHADER).wrap_gl()?;
      let fragment_shader = gl.create_shader(FRAGMENT_SHADER).wrap_gl()?;

      let position = mercantile::LngLat {
        lng: 40.7571418,
        lat: -73.9805655,
      };
      gl.shader_source(
        vertex_shader,
        &format!(
          r"#version 300 es

          uniform highp mat4 proj;
          uniform float zoom_level;
 
          layout (location = 0) in vec2 a_pos;
          void main() {{
            gl_Position = proj * vec4(a_pos, 1.0, 1.0);
          }}"
        ),
      );
      gl.compile_shader(vertex_shader);
      check_compile_status(vertex_shader, "vertex shader")?;
      gl.attach_shader(program, vertex_shader);

      gl.shader_source(
        fragment_shader,
        r"#version 300 es

        uniform highp vec4 fill_color;
        out highp vec4 fragColor;
        void main() {
          fragColor = fill_color;
        }",
      );
      gl.compile_shader(fragment_shader);
      check_compile_status(fragment_shader, "fragment shader")?;
      gl.attach_shader(program, fragment_shader);

      gl.link_program(program);
      if !gl.get_program_link_status(program) {
        bail!("[program] {}", gl.get_program_info_log(program))
      }

      Ok(program)
    }
  }

  fn render(&self, parameters: &Parameters) {
    let gl = &self.gl;
    let graphics = self
      .graphics
      .as_ref()
      .expect("graphics was removed prematurely");

    graphics.render(gl, parameters).expect("failed to");
  }

  fn context_lost(&mut self) {
    self.program = None;
    error!("context lost...");
  }

  fn cleanup(self) {
    use glow::*;
    unsafe {
      if let Some(graphics) = self.graphics {
        graphics.cleanup(&self.gl);
      }
    }
  }
}

trait GlResult<T> {
  fn wrap_gl(self) -> eyre::Result<T>;
}

impl<T> GlResult<T> for Result<T, String> {
  fn wrap_gl(self) -> eyre::Result<T> {
    self.map_err(|error| eyre::Error::msg(error))
  }
}

use impl_details::Parameters;
use zerocopy::IntoBytes;
mod impl_details {
  use std::{
    panic::PanicHookInfo,
    sync::{Mutex, Once},
  };

  use tracing::{Level, debug};
  use tracing_logcat::{LogcatMakeWriter, LogcatTag};
  use tracing_subscriber::fmt::format::Format;

  use crate::transit::android::TransitLayer;

  fn setup_logging() {
    let tag = LogcatTag::Fixed("TransitLines-Rust".to_owned());
    let writer = LogcatMakeWriter::new(tag).expect("Failed to initialize logcat writer");
    tracing_subscriber::fmt()
      .event_format(Format::default().with_level(false).without_time())
      .with_writer(writer)
      .with_ansi(false)
      .with_max_level(Level::TRACE)
      .init();

    std::panic::set_hook(Box::new(panic_hook));
  }

  fn panic_hook(info: &PanicHookInfo) {
    tracing::error!("{info}")
  }
  static LOGGING_SETUP: Once = Once::new();

  static LAYER: Mutex<Option<TransitLayer>> = Mutex::new(None);
  #[unsafe(export_name = "mapCustomLayerInitialize")]
  extern "C" fn initialize() {
    LOGGING_SETUP.call_once(setup_logging);

    if let Some(_) = LAYER.lock().expect("poisoned").replace(TransitLayer::new()) {
      panic!("failed to cleanup previous layer!")
    }
  }

  #[derive(Debug)]
  #[repr(C)]
  pub struct Parameters {
    pub width: f64,
    pub height: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub zoom: f64,
    pub bearing: f64,
    pub pitch: f64,
    pub field_of_view: f64,
    pub projection_matrix: [f64; 16],
  }

  #[unsafe(export_name = "mapCustomLayerRender")]
  unsafe extern "C" fn render(params: *const Parameters) {
    let params = unsafe { &*params };

    let guard = LAYER.lock().expect("poisoned");
    if let Some(layer) = guard.as_ref() {
      layer.render(params);
    } else {
      unreachable!("attempted to render when no layer was available")
    }
  }

  #[unsafe(export_name = "mapCustomLayerContextLost")]
  unsafe extern "C" fn context_lost() {
    let mut guard = LAYER.lock().expect("poisoned");
    debug!("context lost");
    if let Some(layer) = guard.as_mut() {
      layer.context_lost();
    } else {
      unreachable!("attempted to render when no layer was available")
    }
  }
  #[unsafe(export_name = "mapCustomLayerDeinitialize")]
  unsafe extern "C" fn deinitialize() {
    let mut guard = LAYER.lock().expect("poisoned");
    debug!("deinitialize");
    if let Some(layer) = guard.take() {
      layer.cleanup();
    } else {
      unreachable!("attempted to render when no layer was available")
    }
    debug!("deinitialized");
  }
}
