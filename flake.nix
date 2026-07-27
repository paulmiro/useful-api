{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };

  nixConfig = {
    extra-substituters = [ "https://useful-api.cachix.org?priority=10" ];
    extra-trusted-public-keys = [
      "useful-api.cachix.org-1:vlsTmRlyPE64g57Ti+lBXwTHORQKRR2WmpcyGlv5LnI="
    ];
  };

  outputs =
    { flake-parts, crane, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./module.nix
        ./vm-test.nix
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem =
        { pkgs, ... }:
        let
          craneLib = crane.mkLib pkgs;
          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            strictDeps = true;
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
            ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          useful-api = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              meta.mainProgram = "useful-api";
            }
          );
        in
        {
          devShells = {
            default = craneLib.devShell {
              packages =
                (with pkgs; [
                ])
                ++ commonArgs.nativeBuildInputs
                ++ commonArgs.buildInputs;
              inherit (commonArgs) PKG_CONFIG_PATH;
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };
          };

          packages.default = useful-api;
        };
    };
}
