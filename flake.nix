{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-21.05";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, cargo2nix, flake-utils, rust-overlay, ... }:

    # Build the output set for each default system and map system sets into
    # attributes, resulting in paths such as:
    # nix build .#packages.x86_64-linux.<name>
    flake-utils.lib.eachDefaultSystem (system:

      # let-in expressions, very similar to Rust's let bindings.  These names
      # are used to express the output but not themselves paths in the output.
      let

        # create nixpkgs that contains rustBuilder from cargo2nix overlay
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import "${cargo2nix}/overlay")
                      rust-overlay.overlay];
        };

        # create the workspace & dependencies package set
        rustPkgs = pkgs.rustBuilder.makePackageSet' {
          rustChannel = "1.57.0";
          packageFun = import ./Cargo.nix;
          packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "clipboard";
              overrideAttrs = drv: {
                propagatedNativeBuildInputs = drv.propagatedNativeBuildInputs or [ ] ++ [
                  pkgs.xorg.libxcb.dev
                ];
              };
            })
          ];
        };

        workspaceShell = rustPkgs.workspaceShell {
          buildInputs = with pkgs; [
            cargo-edit
            cargo-expand
            cargo-outdated
            cargo-watch
            rust-analyzer
            lldb
            xorg.libxcb

            nixpkgs-fmt
          ];

          nativeBuildInputs = with pkgs; [
            rust-bin.stable.latest.default
          ];
        };

      in rec {
        # this is the output (recursive) set (expressed for each system)

        packages = {
          tyt = (rustPkgs.workspace.tyt {}).bin;
        };

        # nix develop
        devShell = workspaceShell;

        # nix build
        defaultPackage = packages.tyt;

        # nix run
        defaultApp = packages.tyt;
      }
    );
}

