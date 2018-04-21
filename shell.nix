with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "ot";
  buildInputs = [
    rustup
    bashInteractive
  ];
}

