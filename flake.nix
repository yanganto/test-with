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
        rust = pkgs.rust-bin.nightly."2022-03-30".default;
        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          cargo install dependency-refresh
          dr ./Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 0
          fi
          exit 1
        '';
        featureTestScript = pkgs.writeShellScriptBin "feature-test" ''
          cargo run --no-default-features --features=net --example=net
          cargo run --no-default-features --features=user --example=user
          cargo run --no-default-features --features=resource --example=resource
          cargo install cargo-hack
          cargo hack test --examples
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust
            openssl
            pkg-config

            publishScript
            featureTestScript
            updateDependencyScript
          ];
          SAYING = ''
            The value of a man resides in what he gives
            and not in what he is capable of receiving.'';
        };
      }
    );
}
