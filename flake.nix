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
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.package.version;
        name = cargoToml.package.name;
      in
      rec {
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
          shellHook = ''
            if [ ! -f db/oneroster.db ]; then
              sqlite3 db/oneroster.db < db/schema.sql
              sqlite3 db/oneroster.db < db/init.sql
            fi
          '';

        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = name;
          version = version;
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
        packages.docker = pkgs.dockerTools.buildImage {
          name = name;
          tag = version;
          copyToRoot = pkgs.buildEnv {
            name = name;
            paths = [
              packages.default
              pkgs.dockerTools.caCertificates
              pkgs.dockerTools.fakeNss
            ];
          };
          config.EntryPoint = [ "/bin/oneroster" ];
        };

        apps.dockerPush = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "dockerPush" ''
            set -eu
            nix build .#docker -o ${name}
            REPO="git.bgw.dev/bgw/${name}:${version}"
            ${pkgs.skopeo}/bin/skopeo copy \
            --insecure-policy \
            --dest-creds "bgw:$CI_PACKAGE_WRITE" \
            docker-archive:${name} \
            docker://$REPO
          '';
        };

      }
    );
}
