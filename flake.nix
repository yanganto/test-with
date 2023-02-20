{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    dependency-refresh.url = "github:yanganto/dependency-refresh";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils, dependency-refresh }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable."1.64.0".default;
        dr = dependency-refresh.defaultPackage.${system};

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr ./Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 1
          fi
        '';
        featureTestScript = pkgs.writeShellScriptBin "feature-test" ''
          cargo run --no-default-features --features=http --example=http
          cargo run --no-default-features --features=icmp --example=icmp
          cargo run --no-default-features --example=tcp
          cargo run --no-default-features --features=net --example=http
          cargo run --no-default-features --features=net --example=icmp
          cargo run --no-default-features --features=user --example=user
          cargo run --no-default-features --features=resource --example=resource
          cargo run --no-default-features --features=executable --example=executable
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

            dr
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
