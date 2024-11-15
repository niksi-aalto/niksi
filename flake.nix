{
  description = "Niksi";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      inherit (pkgs) lib;

      craneLib = crane.mkLib pkgs;

      src = ./.;

      commonArgs = {
        inherit src;
        strictDeps = true;

        buildInputs = lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      niksi-cli = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });

      niksi-lib = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });
    in {
      checks = {
        inherit niksi-cli niksi-lib;
      };

      packages.default = niksi-cli;

      apps.default = flake-utils.lib.mkApp {
        drv = niksi-cli;
      };

      devShells.default = craneLib.devShell {
        checks = self.checks.${system};
      };
    });
}
