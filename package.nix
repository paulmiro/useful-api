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

  cargoHash = "sha256-Zo31jZH4XmwVF0VPhc2rmklkcqqyorRZwkbygn7KyUk=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];
}
