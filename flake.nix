{
  description = "Mensa UPB scraper and API";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {

      packages.${system}.default = pkgs.callPackage ./default.nix { };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          devenv
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          sqlx-cli
        ];
        env.RUST_SRC_PATH = pkgs.rust.packages.stable.rustPlatform.rustLibSrc;
      };

    };
}
