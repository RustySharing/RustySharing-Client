{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [ 
    rustc 
    cargo
    rustfmt
    rustPackages.clippy
    gcc 
    protobuf
    pkg-config
    glib
    graphene
    gtk4
    libadwaita
    ];
  RUST_BACKTRACE = 1;

}
