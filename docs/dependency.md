# Dependency Resolution of Packages

This file describes how `rapt2` resolves dependencies of packages.

## Glossary

### installation of packages

`rapt2` requests installtion of packages to `dpkg`. `dpkg` installs packages in two steps: **extraction** and **configuration**. This two operations can be done separately by `dpkg --unpack` and `dpkg --configure`, and can be done consecutively by `dpkg -i`.

### `Depends`

There are two types of dependencies, `Depends` and `Pre-Depends`. `Depends` is a weak dependency. A package which has `Depends` field is called *depending-on package*. Packages named in `Depends` field are called *depended-on package*.

If weak dependency is required by `Depends` field, depended-on packages must be configured before configuration of depending-on packages. It is allowed that depending-on packages are extracted before configuration of depended-on packages.

### `Pre-Depends`

`Pre-Depends` requires strong dependency compared to `Depends`. If strong depedency is required, depended-on packages must be configured before **extraction** of depending-on packages. It is NOT allowed even to extract depending-on packages before configuration of depended-on packages are configured.

## Cyclic Dependency

Packages are allowed to have cyclic dependencies. Suppose package `A` has dependency on `B`, and `B` also has dependency on `A`. There is cyclic dependency among `A` and `B`.

This is not documented (as far as I know), but only weak dependency can have cyclic dependency. (I guess) strong dependency would have below limitaion:

- **1.1**: strong dependency CAN'T have cyclic structure.
  - Suppose `A` strongly depends on `B`, `B` on `A`. It is not allowed.
- **1.2**: when strong dependency is regarded as weak dependency, entire dependency graph  (of weak deps) can't have *strongly connected component* which contains strong dependency.
  - Suppose `A` strongly depends on `B`, `B` weakly depends on `C`, and `C` weakly depends on `A`. It wouldn't be allowed.
  - This is not confirmed.


## Dependency Resolution

`rapt2` do below methods to resolve dependencies:

1. Regard `Pre-Depends` as `Depends` and construct directed-graph based on `Depends` field.

2. This graph can have strongly connected component. So `rapt2` does decomposition of *strongly-connected components`. It makes the graph DAG (Directed Acyclic Graph).

3. Do topological sort.

4. Split sorted dependencies into slices(layers). Each layer can have at most one pre-depended-on package. If the layer contains pre-depended-on package, it should be placed at the begining of the layer.

5. For each layer, extract all packages. After extraction, configure all packages.


### notes of thsi steps

- If **1.2** is true, packages inside a same strongly-connected component don't have strong dependency on each other.

- This means that sorted packages of step3 don't have reverse directed strong dependency (though weak dependency can have reverse direction).

- If so, above dependency resolution strategy would work 100% correctly.
