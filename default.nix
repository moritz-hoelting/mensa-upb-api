{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  name = "mensa-upb-api";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    mainProgram = "mensa-upb-api";
    description = "Tools for fetching the API of the canteens of Paderborn University and exposing it as a more user-friendly API";
    license = lib.licenses.mit;
  };
}
