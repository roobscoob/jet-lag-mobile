android studio (winget install Google.AndroidStudio)
  android sdk
  android ndk (add ANDROID_NDK_ROOT env var)
rust (winget install rustup, only aarch64-linux-android target currently)
  msvs build tools + c++ workload
  cargo-ndk for rust-analyzer (cargo install cargo-ndk)
    copy .vscode/settings.template.json to .vscode/settings.json
    replace extraEnv's value with the output of cargo ndk-env -t arm64-v8a --json
cmake (winget install Kitware.CMake)
ninja (winget install Ninja-build.Ninja)
