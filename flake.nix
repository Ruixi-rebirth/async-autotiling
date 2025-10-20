{
  description = "async-autotiling flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        async-autotiling = pkgs.rustPlatform.buildRustPackage {
          pname = "async-autotiling";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      in
      {
        packages = {
          default = async-autotiling;
          async-autotiling = async-autotiling;
        };

        apps = {
          default = {
            type = "app";
            program = "${async-autotiling}/bin/async-autotiling";
          };
          async-autotiling = {
            type = "app";
            program = "${async-autotiling}/bin/async-autotiling";
          };
        };
      }
    );
}
