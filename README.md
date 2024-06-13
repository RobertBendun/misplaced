# Misplaced - build system to use when you loose cargo

Playing around with simple build system for Rust.
Inspired by [nobuild](https://github.com/tsoding/nobuild).

## Usage:

First build your build system with:

```
rustc build.rs -o build
```

Then let it rebuild itself when needed:

```
./build
```

