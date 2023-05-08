{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-filter.url = "github:numtide/nix-filter";
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
        ...
      }: {
        devShells.default = with pkgs;
          mkShell {
            # Shell with CC
            # name = "nh-dev";
            RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
            packages = [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer-unwrapped
            ];
            RUST_BACKTRACE = "full";
          };
      };
    };
}
