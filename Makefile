DOCKER_IMAGE_NAME="rapt2-dev"
DOCKER_CONTAINER_NAME="rapt2-tmp"

# construct a clea ndocker image and run container with pwd bind-mounted.
docker:
	cargo build --release
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

.PHONY: docker
