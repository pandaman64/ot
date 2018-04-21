with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "ot";
  buildInputs = [
    rustup
    bashInteractive
    openssl
    pkgconfig
    zlib
    cmake
  ];
  shellHook = ''
    export PATH=$PWD/.cargo/bin:$PATH
  '';
}

