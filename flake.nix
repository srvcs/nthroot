{
  description = "srvcs-nthroot: arithmetic: nth root of value (alias of root)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        version = "0.1.0";
        rustToolchain = pkgs.rust-bin.stable."1.96.0".default.override {
          extensions = [ "clippy" "rustfmt" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in {
        packages = {
          default = rustPlatform.buildRustPackage {
            pname = "srvcs-nthroot";
            inherit version;
            src = ./.;
            cargoHash = "sha256-3IgGcO70NN0sxyKuV7ZlrSWUk4Q1bKOnxeUSiX0hglA=";
          };
        } // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          container = pkgs.dockerTools.buildLayeredImage {
            name = "srvcs-nthroot";
            tag = "latest";
            config = {
              Entrypoint = [ "${self.packages.${system}.default}/bin/srvcs-nthroot" ];
              ExposedPorts = { "8080/tcp" = { }; };
              User = "65534:65534";
              Labels = {
                "org.opencontainers.image.title" = "srvcs-nthroot";
                "org.opencontainers.image.description" = "Nth-root orchestrator: a thin alias of srvcs-root, delegating the nth root of value entirely to srvcs-root.";
                "org.opencontainers.image.version" = version;
                "org.opencontainers.image.revision" = self.rev or "dev";
                "org.opencontainers.image.source" = "https://github.com/srvcs/nthroot";
                "org.opencontainers.image.licenses" = "Apache-2.0";
              };
            };
          };
        };

        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain pkgs.syft ];
        };
      });
}
