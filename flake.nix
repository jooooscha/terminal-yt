{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      naersk-lib = naersk.lib."${system}";
    in rec {
      # `nix build`
      packages.tyt = naersk-lib.buildPackage {
        pname = "tyt";
        root = ./.;
        buildInputs = with pkgs; [ python310 pkgconfig openssl xorg.libxcb ];
        nativeBuildInputs = with pkgs; [ python310 xorg.libxcb];
      };
      defaultPackage = packages.tyt;

      # `nix run`
      apps.tyt = utils.lib.mkApp {
        drv = packages.tyt;
      };
      defaultApp = apps.tyt;

      # `nix develop`
      # devShell = pkgs.mkShell {
      #   nativeBuildInputs = with pkgs; [ rustc cargo ];
      # };
    });
}
