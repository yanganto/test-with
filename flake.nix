{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable.latest.default;
        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cd $1
          cargo login $2
          cargo publish
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust

            publishScript
          ];
          SAYING = ''
            The value of a man resides in what he gives
            and not in what he is capable of receiving.'';
        };
      }
    );
}
