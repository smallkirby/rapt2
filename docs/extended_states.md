# extended_states

`/var/lib/apt/extended_states` is used by original `apt`. It holds below package information insalled via `apt`.

- package name
- architecture
- whether automatically installed or not

**It seems that original `apt` doesn't check upgradability of automatically installed packages when `apt update`.**
