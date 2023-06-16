{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

      in
      {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            jq.lib
            openssl.dev
            sqlite
            oniguruma
            pkgconfig
          ];
          JQ_LIB_DIR = "${pkgs.jq.lib}";
          DATABASE_URL = "sqlite:db/oneroster.db";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "oneroster";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgs.pkgconfig
            pkgs.openssl.dev
            pkgs.sqlite
          ];
          buildInputs = [
            pkgs.jq.lib
            pkgs.oniguruma
          ];
          preConfigure = ''
            sqlite3 db/oneroster.db < db/schema.sql
            sqlite3 db/oneroster.db < db/init.sql
          '';
          doCheck = false;
          JQ_LIB_DIR = "${pkgs.jq.lib}";
          DATABASE_URL = "sqlite:db/oneroster.db";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      }
    );
}
