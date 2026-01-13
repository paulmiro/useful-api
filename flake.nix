{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { flake-parts, crane, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
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

            gemini = craneLib.devShell {
              packages =
                (with pkgs; [
                  gemini-cli
                ])
                ++ commonArgs.nativeBuildInputs
                ++ commonArgs.buildInputs;
              inherit (commonArgs) PKG_CONFIG_PATH;
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };
          };

          packages.default = useful-api;
          packages.useful-api = useful-api;
        };
    };
}
