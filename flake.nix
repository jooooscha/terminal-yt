{
  inputs = {
      nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let

      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };

    in {

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "tyt";
        version = "2.0.5";
        src = pkgs.lib.cleanSource ./.;

        cargoHash = "sha256-ClrNmvnXAyco3ltGPflzi5q/zmNC24V4mwBhDs9SRGw=";

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        meta = {};
      };
    };
}
