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
    devenv.url = "github:cachix/devenv";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {nixpkgs.follows = "nixpkgs";};
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    devenv,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      inherit (pkgs) lib;

      craneLib = crane.mkLib pkgs;

      commonArgs = {
        src = ./.;
        strictDeps = true;

        buildInputs = lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };

      unfilteredRoot = ./.;
      src = lib.fileset.toSource {
        root = unfilteredRoot;
        fileset = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          (craneLib.fileset.commonCargoSources ./cli)
          (craneLib.fileset.commonCargoSources ./lib)
          ./cli/assets
        ];
      };

      niksi = craneLib.buildPackage (commonArgs
        // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          cargoExtraArgs = "-p cli";
          inherit src;
        });
    in {
      checks = {
        inherit niksi;
      };

      packages.default = niksi;

      apps.default = flake-utils.lib.mkApp {
        drv = niksi;
      };

      devShells.default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ({
            pkgs,
            config,
            ...
          }: {
            packages = with pkgs; [skopeo];

            languages.rust = {
              enable = true;
              channel = "stable";
            };

            env = {
              LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_latest.libclang.lib];
              BINDGEN_EXTRA_CLANG_ARGS =
                (builtins.map (a: ''-I"${a}/include"'') [
                  pkgs.glibc.dev
                ])
                ++ [
                  ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
                  ''-I"${pkgs.glib.dev}/include/glib-2.0"''
                  ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
                ];
            };
          })
        ];
      };
    });
}
