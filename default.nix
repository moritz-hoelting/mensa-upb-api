{ rustPlatform }:
rustPlatform.buildRustPackage {
  name = "mensa-upb-api";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
