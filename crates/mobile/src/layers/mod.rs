mod oob {
  #[cfg(target_os = "android")]
  pub mod android;
}
pub mod transit {
  //#[cfg(target_os = "android")]
  //pub mod android;
}

mod android {
  use std::{panic::PanicHookInfo, ptr, sync::Once};

  use glam::DMat4;
  use palette::white_point::E;
  use tracing::Level;
  use tracing_logcat::{LogcatMakeWriter, LogcatTag};
  use tracing_subscriber::fmt::format::Format;

  use crate::layers::oob::android::OutOfBoundsLayer;

  fn setup_logging() {
    static LOGGING_SETUP: Once = Once::new();

    LOGGING_SETUP.call_once(|| {
      let tag = LogcatTag::Fixed("JetLag-Rust".to_owned());
      let writer = LogcatMakeWriter::new(tag).expect("Failed to initialize logcat writer");
      tracing_subscriber::fmt()
        .event_format(Format::default().with_level(false).without_time())
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(Level::TRACE)
        .init();

      std::panic::set_hook(Box::new(panic_hook));
    })
  }

  fn panic_hook(info: &PanicHookInfo) {
    tracing::error!("{info}")
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
    pub projection_matrix: DMat4,
  }

  pub trait CustomLayer: Sized {
    fn new() -> eyre::Result<Self>;
    fn render(&mut self, parameters: &Parameters) -> eyre::Result<()>;
    fn context_lost(&mut self);
    fn cleanup(self);
  }

  #[repr(C)]
  struct CustomLayerVTable {
    pub initialize: extern "C" fn(*mut CustomLayerVTable),
    pub render: extern "C" fn(*mut CustomLayerVTable, *const Parameters),
    pub context_lost: extern "C" fn(*mut CustomLayerVTable),
    pub deinitialize: extern "C" fn(*mut CustomLayerVTable),
    pub boxed_value: *mut (),
  }

  extern "C" fn initialize<T: CustomLayer>(vtable: *mut CustomLayerVTable) {
    (unsafe { &mut *vtable }).boxed_value =
      Box::into_raw(Box::new(T::new().expect("failed to construct type"))).cast()
  }

  extern "C" fn render<T: CustomLayer>(
    vtable: *mut CustomLayerVTable,
    parameters: *const Parameters,
  ) {
    let value = unsafe { &mut *(*vtable).boxed_value.cast::<T>() };
    value
      .render(unsafe { &*parameters })
      .expect("failed to render a frame")
  }

  extern "C" fn context_lost<T: CustomLayer>(vtable: *mut CustomLayerVTable) {
    let value = unsafe { &mut *(*vtable).boxed_value.cast::<T>() };
    value.context_lost();
  }

  extern "C" fn deinitialize<T: CustomLayer>(vtable: *mut CustomLayerVTable) {
    let value = unsafe { Box::from_raw((*vtable).boxed_value.cast::<T>()) };
    value.cleanup();
  }

  const fn custom<T: CustomLayer>() -> CustomLayerVTable {
    CustomLayerVTable {
      initialize: initialize::<T>,
      render: render::<T>,
      context_lost: context_lost::<T>,
      deinitialize: deinitialize::<T>,
      boxed_value: ptr::null_mut(),
    }
  }

  // to allow it to remain in a static
  unsafe impl Sync for CustomLayerVTable {}

  static OUT_OF_BOUNDS_LAYER: CustomLayerVTable = custom::<OutOfBoundsLayer>();

  #[unsafe(export_name = "fetchCustomLayerVtable")]
  extern "C" fn fetch_custom_layer_vtable(kind: u32) -> *const CustomLayerVTable {
    setup_logging();
    tracing::info!("fetching custom layer vtable: {kind}");
    match kind {
      0 => &raw const OUT_OF_BOUNDS_LAYER,
      _ => {
        panic!("picked an invalid layer")
      }
    }
  }

  pub mod gl {
    use std::{
      sync::LazyLock,
      thread::{self, ThreadId},
    };

    use khronos_egl::DynamicInstance;

    pub fn get_gl_context() -> &'static glow::Context {
      static DYNAMIC: LazyLock<glow::Context> = LazyLock::new(|| unsafe {
        let instance = DynamicInstance::load().expect("failed to obtain egl instance");
        glow::Context::from_loader_function(move |str| {
          instance
            .get_proc_address(str)
            .map(|x| x as *const _)
            .unwrap_or_default()
        })
      });

      static CONTEXT_THREAD: LazyLock<ThreadId> = LazyLock::new(|| thread::current().id());

      if *CONTEXT_THREAD != thread::current().id() {
        panic!("accessed gl context on a different thread from normal")
      }

      &DYNAMIC
    }

    pub trait GlResult<T> {
      fn wrap_gl(self) -> eyre::Result<T>;
    }

    impl<T> GlResult<T> for Result<T, String> {
      fn wrap_gl(self) -> eyre::Result<T> {
        self.map_err(|error| eyre::Error::msg(error))
      }
    }
  }
}
