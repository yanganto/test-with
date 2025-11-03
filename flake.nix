{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    dependency-refresh = {
      url = "github:yanganto/dependency-refresh";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils, dependency-refresh }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        dr = dependency-refresh.defaultPackage.${system};

        publishScript = pkgs.writeShellScriptBin "crate-publish" ''
          cargo login $1
          cargo publish
        '';
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr -p ./Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 1
          fi
        '';
        featureTestScript = pkgs.writeShellScriptBin "feature-test" ''
          set -e
          cargo run --no-default-features --features=http --example=http
          cargo run --no-default-features --features=icmp --example=icmp
          cargo run --no-default-features --example=tcp
          cargo run --no-default-features --features=net --example=http
          cargo run --no-default-features --features=net --example=icmp
          cargo run --no-default-features --features=user --example=user
          cargo run --no-default-features --features=resource --example=resource
          cargo run --no-default-features --features=executable --example=executable
          cargo run --no-default-features --features=timezone --example=timezone
          cargo install cargo-hack
          cargo hack test --examples

          # runtime ignore example
          cd examples/runner
          cargo run --example test
          cargo run --example mock
          cargo run --example mock2
          cargo run --example mix
          cargo run --example tokio
        '';
        cargoTomlConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      with pkgs;
      {
        devShells =  {
          default = mkShell {
            buildInputs = [
              rust-bin.stable.${cargoTomlConfig.package.rust-version}.minimal
              openssl
              pkg-config
            ];
          };
          
          ci = mkShell {
            buildInputs = [
              rust-bin.stable.${cargoTomlConfig.package.rust-version}.default
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
        };
      }
    );
}
