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
        devShell = pkgs.mkShell
          {
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

        packages = { };
        #packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;

        #packages.x86_64-linux.default = self.packages.x86_64-linux.hello;
      }
    );
}
