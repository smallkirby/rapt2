DOCKER_IMAGE_NAME="rapt2-dev"
DOCKER_CONTAINER_NAME="rapt2-tmp"

update:
	cargo run -- \
		--dpkg-dir "/var/lib/dpkg" \
		--source-dir "./tests/resources/sources" \
		--list-dir "./rapt2/lists" \
		update

update-deb:
	cargo run -- \
		--dpkg-dir "./rapt2/" \
		--source-dir "./tests/resources/sources" \
		--list-dir "./rapt2/lists" \
		update

docker:
	cargo build
	docker build -t $(DOCKER_IMAGE_NAME) .
	docker container run -it \
		-w "/home/user/rapt2" \
		--name $(DOCKER_CONTAINER_NAME) \
		--rm $(DOCKER_IMAGE_NAME) \
		/bin/bash

.PHONY: update update-deb docker