{
  description = "Niksi";

  inputs = {
    nixkpgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";

    devenv.url = "github:cachix/devenv";
  };

  nixConfig = {
    extra-trusted-public-keys = [
      "eigenvalue.cachix.org-1:ykerQDDa55PGxU25CETy9wF6uVDpadGGXYrFNJA3TUs="
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
    ];
    extra-substituters = [
      "https://eigenvalue.cachix.org"
      "https://devenv.cachix.org"
    ];
    allow-import-from-derivation = true;
  };

  outputs = inputs @ {
    nixpkgs,
    flake-parts,
    crate2nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
      ];

      imports = [
        inputs.devenv.flakeModule
        ./nix/rust-overlay
      ];

      perSystem = {
        system,
        pkgs,
        lib,
        inputs',
        ...
      }: let
        cargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
          name = "niksi";
          src = ./.;
        };
      in rec {
        imports = [
          ./nix/devenv
        ];

        checks = {
          lib = cargoNix.workspaceMembers.lib.build.override {
            runTests = true;
          };
          cli = cargoNix.workspaceMembers.cli.build.override {
            runTests = true;
          };
        };

        packages = {
          niksi = cargoNix.workspaceMembers.cli.build;
          default = packages.niksi;

          inherit (pkgs) rust-toolchain;

          rust-toolchain-versions = pkgs.writeScriptBin "rust-toolchain-versions" ''
            ${pkgs.rust-toolchain}/bin/cargo --version
            ${pkgs.rust-toolchain}/bin/rustc --version
          '';
        };

        apps = {
          niksi = {
            type = "app";
            program = "${packages.niksi}/bin/niksi";
          };
          default = apps.niksi;
        };
      };
    };
}
