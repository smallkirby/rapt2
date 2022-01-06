# rapt2

<video controls width="100%" width="100%" autoplay loop="true" src="https://user-images.githubusercontent.com/47111091/148347839-04d2075c-5e16-4ad9-a43d-e5a406664f0d.mp4" type="video/mp4">
Vide play not supported.
</video>

**`rapt2`** is a *kawaii* toy implementation of `apt`(Debian Package Manager) written in Rust (Original `apt` is written in C++).

**`rapt2`** is simplified, made of only 3K LOC of Rust (as of 2021.12.30). Hence, it has limitation compared to original one. But no problem. *kawaii* is justice.

- `update`

![update](/img/update.png)

- `install`

![install](/img/install.png)

- `dep`

![dep](/img/dep.png)

- `list`

![list](/img/list.png)

- `upgrade`

![upgrade](/img/upgrade.png)

- `clean` / `purge`

## Development

- It is recommended to use docker environment to prevend your native environments are collapsed.
- Just exec `make docker` in the top of this repository. It builds and starts a container. The container is Ubuntu based clean environment, where only pwd is bind-mounted.

## TODOs

- multi-threaded download and installation
- caching
- more fast format of cache files
- commands implemented in original `apt`
- etc
