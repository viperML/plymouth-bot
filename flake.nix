{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    fenix,
    ...
  }:
    inputs.flake-parts.lib.mkFlake {inherit self;} {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
        "x86_64-darwin"
      ];

      flake.overlays.default = final: prev: let
        inherit (nixpkgs) lib;
        variant = "stable";
        buildSystem = final.stdenv.buildPlatform.system;
        targetConfig = final.stdenv.targetPlatform.config;
        canBeStatic = lib.strings.hasSuffix "musl" targetConfig;
      in {
        _toolchain = with fenix.packages.${buildSystem};
          combine [
            (fenix.packages.${buildSystem}.${variant}.withComponents [
              "rustc"
              "cargo"
            ])
            targets.${targetConfig}.${variant}.rust-std
          ];

        plymouth-bot =
          (final.makeRustPlatform {
            cargo = final._toolchain;
            rustc = final._toolchain;
          })
          .buildRustPackage {
            src = inputs.nix-filter.lib {
              root = ./.;
              include = [
                (inputs.nix-filter.lib.inDirectory "src")
                "Cargo.toml"
                "Cargo.lock"
                "build.rs"
              ];
            };
            name = "plymouth-bot";
            cargoLock.lockFile = ./Cargo.lock;
            target = targetConfig;
            CARGO_BUILD_RUSTFLAGS =
              if canBeStatic
              then "-C target-feature=+crt-static"
              else "";
          };

        _toolchain_dev = fenix.packages.${buildSystem}.${variant}.withComponents [
          "rustc"
          "cargo"
          "rust-src"
          "clippy"
          "rustfmt"
          "rust-analyzer"
        ];
      };

      perSystem = {
        system,
        pkgs,
        config,
        ...
      }: {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [self.overlays.default];
        };

        packages = {
          default = pkgs.plymouth-bot;
          static = pkgs.pkgsCross.aarch64-multiplatform-musl.plymouth-bot;
        };

        devShells.default = with pkgs;
          mkShell {
            RUST_SRC_PATH = "${pkgs._toolchain_dev}/lib/rustlib/src/rust/library";
            packages = [
              pkgs._toolchain_dev
            ];
          };

        legacyPackages = pkgs;
      };
    };
}
