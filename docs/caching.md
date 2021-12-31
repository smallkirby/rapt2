# caching

`rapt2` uses below caching strategy to check update of `Packages` file.

# check update of `Packages` file itself

- Put `If-Modified-Since` header in GET request of `InRelease` file.
- If there is update of `InRelease` file, check MD5 hash of `Packages` file in `InRelease` response.
- If the two hashes of known `Packages` and new `Packages` differ, fetch full `Packages` and update list DB.
