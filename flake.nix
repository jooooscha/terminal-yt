{
  inputs.easy.url = "github:jooooscha/easy-flake";
  # inputs.easy.url = "path:/home/joscha/main/programming/nix/easy-flake";

  outputs = { easy, ...}:
    easy.rust.x86_64-linux.env {
      ssl = true;
      root = ./.;
      name = "tyt";
    };
}
