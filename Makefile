update:
	cargo run -- \
		--dpkg-dir "/var/lib/dpkg" \
		--source-dir "./tests/resources/sources" \
		--list-dir "./rapt2/lists" \
		update

.PHONY: update