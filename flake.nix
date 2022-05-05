{
  inputs.easy.url = "github:jooooscha/easy-flake";

  outputs = { easy, nixpkgs, ...}:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in easy.rust {
      ssl = true;
      inputs = with pkgs; [
        xorg.libxcb
      ];
    };
}
# {
#   inputs = {
#     utils.url = "github:numtide/flake-utils";
#     naersk.url = "github:nix-community/naersk";
#   };

#   outputs = { self, nixpkgs, utils, naersk }:
#     utils.lib.eachDefaultSystem (system: let
#       pkgs = nixpkgs.legacyPackages."${system}";
#       naersk-lib = naersk.lib."${system}";
#     in rec {
#       # `nix build`
#       packages.my-project = naersk-lib.buildPackage {
#         pname = "tyt";
#         root = ./.;
#       };
#       defaultPackage = packages.my-project;

#       # `nix run`
#       apps.my-project = utils.lib.mkApp {
#         drv = packages.my-project;
#       };
#       defaultApp = apps.my-project;

#       # `nix develop`
#       devShell = pkgs.mkShell {
#         nativeBuildInputs = with pkgs; [ rustc cargo ];
#       };
#     });
# }
