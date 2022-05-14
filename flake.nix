{
  inputs.easy.url = "github:jooooscha/easy-flake";

  outputs = { easy, ...}:
    easy.rust {
      ssl = true;
      root = ./.;
      name = "tyt";
    };
}
