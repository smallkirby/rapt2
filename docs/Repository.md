# Repository

This file describes file format at debian package repositories (such as `deb http://jp.archive.ubuntu.com/ubuntu/`).

# `Packages` file

| Field | Mandatory | `rapt2` support | Description |
|-----|-----|-----|-----|
|`Description`|o|o|Description of package. Consisting of short and (optional) description.|
|`Size`|?|o|`.deb` package size?|
|`Source`|x|x|Source package name.|
|`Package`|o|o|Name of binary package.|
|`Version`|o|o|Version of binary package. Refer to [`rapt2`'s implementation](/src/package/version.rs) for its structure and ordering.|
|`Prioriy`|△|o|hoge|
|`Architecture`|o|o|unique identifier of architecture / `all` for arch-independent binary / `source` for source package.|
|`Essential`|x|x|`yes` or `no`, indicating package manager would refuse removal of it.|
|`Depends`|x|△|Absolute package dependencies. Circular dependency can happen. It is recommended to unpack all first, then to configure all later. Comma or vertical bar separated.|
|`Pre-Depends`|x|x|Depended-on packages must be configured before depending-on package installation. Circular dependencies are not allowed.|
|`Recommends`|x|x|Strong but optional dependencies.|
|`Suggests`|x|x|Weak dependencies.|
|`Breaks`|x|x|It breaks other packages. Installation is refused til named packages are deconfigured.|
|`Conflicts`|x|x|Stronger restriction than `Breaks`. Even it is refused to configure broken packages while breaking package is `unpacked` state.|
|`Provides`|x|x|Virtual package. The package satisfies dependency requirements instead of named packages.|
|`Replaces`|x|x|Allow overwriting of replaced packages' files.|
|`Enhances`|x|x|Same but the opposite direction of `Suggests`.|
|etc...|?|?|(too lazy to leave a memo)|


# `Sources` file

| Field | Mandatory | `rapt2` support | Description |
|-----|-----|-----|-----|
|`Source`|o|o|Source package name.|
|`Maintainer`|o|o|.|
|`Uploaders`|x|x|.|
|`Section`|△|o|.|
|`Priority`|△|o|.|
|`Standards-Version`|o|x|.|
|`Homepage`|x|o|.|
|etc...|?|?|(too lazy to leave a memo)|


## Mandatory Legends

- `o`: mandatory
- `x`: optional
- `△`: recommended

# References

- https://www.debian.org/doc/debian-policy/#s-binarycontrolfiles
- https://wiki.debian.org/DebianRepository/Format