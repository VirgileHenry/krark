{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
    pkgs.python313
    pkgs.python313Packages.requests
    pkgs.python313Packages.stringcase
  ];
  RUST_BACKTRACE=1;
  TMPDIR="/tmp";
}
