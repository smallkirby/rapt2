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

.PHONY: update update-deb