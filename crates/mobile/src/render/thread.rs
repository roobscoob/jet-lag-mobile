use jet_lag_core::shape::instruction::SdfInstruction;

#[cfg(target_os = "android")]
struct StartParams(khronos_egl::Context);
unsafe impl Send for StartParams {}

pub fn start_render_thread(start_params: StartParams) {
  std::thread::spawn(|| renderer_thread(start_params));
}

enum Messages {
  ReceiveInstructions(Vec<SdfInstruction>),

}

pub fn renderer_thread(start_params: StartParams) {
  tokio::runtime::Builder::new_current_thread()
    .build()
    .expect("failed to create tokio runtime")
    .block_on(async move {});
}
