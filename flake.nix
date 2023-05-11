{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-filter.url = "github:numtide/nix-filter";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.url = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem = {
        system,
        pkgs,
        config,
        inputs',
        ...
      }: let
        proc = pkgs.targetPlatform.uname.processor;
        CARGO_BUILD_TARGET = "${proc}-unknown-linux-musl";
        src = inputs.nix-filter.lib {
          root = inputs.self;
          include = [
            (inputs.nix-filter.lib.inDirectory "src")
            "Cargo.toml"
            "Cargo.lock"
          ];
        };
      in {
        _module.args.pkgs = inputs'.nixpkgs.legacyPackages.extend inputs.rust-overlay.overlays.default;
        legacyPackages = pkgs;

        packages = {
          toolchain-dev = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rustfmt"
            ];
          };
          toolchain-static = pkgs.rust-bin.stable.latest.default.override {
            extensions = [];
            targets = [CARGO_BUILD_TARGET];
          };
          plymouth-bot = pkgs.callPackage ./package.nix {inherit src;};
          plymouth-bot-static = pkgs.callPackage ./package.nix {
            inherit src;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = config.packages.toolchain-static;
              rustc = config.packages.toolchain-static;
              stdenv = pkgs.pkgsStatic.stdenv;
              inherit CARGO_BUILD_TARGET;
              CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
            };
          };
        };

        devShells.default = with pkgs;
          mkShell {
            RUST_SRC_PATH = "${config.packages.toolchain-dev}/lib/rustlib/src/rust/library";
            packages = [
              config.packages.toolchain-dev
              rust-analyzer-unwrapped
              vault
            ];
            RUST_BACKTRACE = "1";
          };
      };
    };
}
