{ nixpkgs ? import <nixpkgs> { }}:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "ba8b70ee098bc5654c459d6a95dfc498b91ff858";
    hash   = "sha256-IKKIXTSYJMmUtE+Kav5Rob8SgLPnfnq4Qu8LyT4gdqQ=";
  };
  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      SDL2
      SDL2_gfx
      rust-bin.stable.latest.default
      rust-analyzer
    ];

    RUST_BACKTRACE = 1;
  }
