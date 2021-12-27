# Lock

Original `apt` and `dpkg` locks specific file for exclusive access control. `rapt2` has to lock these files too.

These lock are acquired `flock` system call.

## `/var/lib/apt/lists/lock`

The lock of this file is acquired before updating list DB. Refer to `/apt-pkg/update.cc` of original apt source.
