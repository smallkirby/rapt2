# Sources.list

Sources.list file has below format:

```sources.list
[archive type] [url] [distro] [components]
```

- `archive type`: `deb` for binary package, `deb-src` for source package.
- `url`: base URL.
- `distro`: Ubuntu distribution.
- `components`: more than 1 components. `main restricted universe multiverse partner contrib stable`. For their meaning, refer to `/src/source/source.rs`.

## Exception?

Source file (seems) can have different format from above one.
Example is: `deb http://download.opensuse.org/repositories/home:/katacontainers:/releases:/x86_64:/master/xUbuntu_20.04/ /`.

From this example, we can guess that:

- `distro` can have arbitrary value.
- `components` can be empty.

In this example, `distro` is `/`. This would be a little hack to navigate all distributions into the same directory. Also, `components` is empty so that the one directory is used.
Above line would be converted to: `http://download.opensuse.org/repositories/home:/katacontainers:/releases:/x86_64:/master/xUbuntu_20.04/`.
