DOCKER_IMAGE_NAME="rapt2-dev"
DOCKER_CONTAINER_NAME="rapt2-tmp"

# do `rapt2 update` using actual dpkg status.
update:
	cargo run -- \
		--dpkg-dir "/var/lib/dpkg" \
		--source-dir "./tests/resources/sources" \
		--list-dir "./rapt2/lists" \
		update

# do `rapt2 update` using self-customized dpkg status.
update-deb:
	cargo run -- \
		--dpkg-dir "./rapt2/" \
		--source-dir "./tests/resources/sources" \
		--list-dir "./rapt2/lists" \
		update

# construct a clea ndocker image and run container with pwd bind-mounted.
docker:
	cargo build
	docker build \
		--build-arg UID=$(shell id -u) \
		--build-arg GID=$(shell id -g) \
		-t $(DOCKER_IMAGE_NAME) .
	docker container run -it \
		-v $(shell pwd):/home/user/rapt2:Z \
		-w "/home/user/rapt2" \
		--name $(DOCKER_CONTAINER_NAME) \
		--rm $(DOCKER_IMAGE_NAME) \
		/bin/bash

.PHONY: update update-deb docker