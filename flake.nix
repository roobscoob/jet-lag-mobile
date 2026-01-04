{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        androidComposite = (
          pkgs.androidenv.composeAndroidPackages {
            includeNDK = true;
            platformVersions = [
              "35"
            ];
            buildToolsVersions = [
              "35.0.0"
            ];
            cmakeVersions = [
              "3.22.1"
            ];
          }
        );
        androidsdk = androidComposite.androidsdk;
        androidndk = androidComposite.ndk-bundle;
      in
      with pkgs;
      {
        formatter = nixfmt-tree;
        inherit inputs;
        devShells.default = pkgsCross.aarch64-multiplatform.mkShell.override { stdenv = pkgsCross.aarch64-multiplatform.llvmPackages.stdenv; } {
          nativeBuildInputs = [
            (pkgs.fenix.combine [
              pkgs.fenix.stable.defaultToolchain
              pkgs.fenix.stable.rust-src
              pkgs.fenix.targets.armv7-linux-androideabi.stable.rust-std
              pkgs.fenix.targets.aarch64-linux-android.stable.rust-std
              pkgs.fenix.targets.x86_64-linux-android.stable.rust-std
              pkgs.fenix.targets.i686-linux-android.stable.rust-std
            ])
            pkg-config
            gradle
            androidsdk
            uv
            cmake
            go
            openssl
          ];
          buildInputs = with pkgsCross.aarch64-multiplatform; [
            openssl
          ];
          ANDROID_HOME = "${androidsdk}/libexec/android-sdk";
          ANDROID_NDK_ROOT = "${androidndk}/libexec/android-sdk/ndk-bundle";
          LD_LIBRARY_PATH = lib.makeLibraryPath (with pkgsCross.aarch64-multiplatform; [
            openssl
          ]);

          shellHook = ''
            unset TMPDIR
            unset TMP
            unset TEMPDIR
            unset TEMP
            pkill -f '.*GradleDaemon.*'
          '';
        };
      }
    );
}
