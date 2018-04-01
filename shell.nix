with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "ot";
  buildInputs = [
    latest.rustChannels.stable.rust
    bashInteractive
  ];
}

