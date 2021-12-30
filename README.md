# rapt2

**`rapt2`** is a *kawaii* toy implementation of `apt`(Debian Package Manager) written in Rust (Original `apt` is written in C++).

**`rapt2`** is simplified, made of only 3K LOC of Rust (as of 2021.12.30). Hence, it has limitation compared to original one. But no problem. *kawaii* is justice.

- `update`

![update](/img/update-2.png)

- `install`

![install](/img/install-1.png)

- `dep`

![dep](/img/dep-1.png)

- `list`

![list](/img/list-1.png)

## Development

- It is recommended to use docker environment to prevend your native environments are collapsed.
- Just exec `make docker` in the top of this repository. It builds and starts a container. The container is Ubuntu based clean environment, where only pwd is bind-mounted.

## TODOs

- multi-threaded download and installation
- caching
- more fast format of cache files
- commands implemented in original `apt`
- etc
