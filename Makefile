dev:
	nix develop

build:
	cargo build

test:
	cargo test

nix:
	nix build

docker:
	nix build .#docker
	docker load -i ./result

database:
	sqlite3 oneroster.db < db/schema.sql
	sqlite3 oneroster.db < db/init.sql

ci:
	act -W .gitea/workflows
