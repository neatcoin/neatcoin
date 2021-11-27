{
  description = "Neatcoin dev environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-21.05";
    rustOverlay.url = "github:oxalica/rust-overlay";
    rustOverlay.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, rustOverlay }: let
    shellEnv = system: let
      pkgs = import nixpkgs { overlays = [ rustOverlay.overlay ]; inherit system; };
      rust-nightly = with pkgs; ((rustChannelOf { channel = "1.56.1"; }).default.override {
        extensions = [ "rust-src" ];
        targets = [ "wasm32-unknown-unknown" ];
      });
    in with pkgs; mkShell {
      buildInputs = [
        clang
        openssl.dev
        pkg-config
        rust-nightly
        wabt
      ] ++ lib.optionals stdenv.isDarwin [
        darwin.apple_sdk.frameworks.Security
      ];

      RUST_SRC_PATH = "${rust-nightly}/lib/rustlib/src/rust/src";
      LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
      PROTOC = "${protobuf}/bin/protoc";
      ROCKSDB_LIB_DIR = "${rocksdb}/lib";
	  RUSTC_BOOTSTRAP = "1";
    };
  in {
    devShell."x86_64-linux" = shellEnv "x86_64-linux";
    devShell."x86_64-darwin" = shellEnv "x86_64-darwin";
  };
}
