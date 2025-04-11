{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }: let
    system = "x86_64-linux";
    overlays = [ (import rust-overlay) ];

    pkgs = import nixpkgs {
      inherit system overlays;
    };

    rustBin = pkgs.pkgsBuildHost.rust-bin;

    rustPkg = rustBin.fromRustupToolchainFile
      ./rust-toolchain.toml;

  in {
    devShell.${system} = with pkgs; mkShell {
      nativeBuildInputs = [
        openssl
        rustPkg
        glib
        pkg-config
      ];

      buildInputs = [
        dbus
        glib

        libxkbcommon
        fontconfig
        wayland
        xwayland

        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXmu
        xorg.libXrandr
      ];

      LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
        with pkgs; lib.makeLibraryPath [
          wayland
          libxkbcommon
          fontconfig
        ] }";
    };
  };
}
