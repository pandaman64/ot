with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "satysfi-playground";
  buildInputs = [
    latest.rustChannels.stable.rust
  ];
}

