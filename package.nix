{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = lib.cleanSource ./.;

  cargoHash = "sha256-9IXQHcghubM9YVbNwPQiiulyj/hcITrGOImn4NYB4kw=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];
}
