version := 0.2.0

build:
	cargo build --release

podman-server:
	podman build --tag oneroster:${version} .

podman-sync:
	podman build --tag oneroster-sync:${version} --file Dockerfile-sync .
