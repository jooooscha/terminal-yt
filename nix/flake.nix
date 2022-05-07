{
  description = "Terminal-YT nix module";

  outputs = { self, nixpkgs }: {
    nixosModule = import ./module.nix;
  };
}
