# Installation guide 

The plugin is written in `Rust`, which is a compiled language, in contrast to `lua` which is interpreted and which
engine is bundled with `neovim`, this creates a few restrictions.  

## Details

`neovim` can also load `.so`s (and I think `.dll`s), dynamically loaded libraries under the plugin-name.
E.g. `nvim_winpick` will be loaded as `nvim_winpick.so`, it understands that it's a plugin if that `.so` 
is placed under `<dir>/lua/nvim_winpick.so`. 
Thus, you can build this plugin manually from source by `cargo b`, go into `target` and find `libnvim_winpick.so` 
(`libnvim_winpick.dylib`, `libnvim_winpick.dll` for mac and windows respectively), 
then copy it to `<some-dir>/lua/nvim_winpick.so` (for linux and mac, `nvim_winpick.dll` for windows), 
then inform `neovim` that the plugin exists by running: `:set rtp+=<some-dir>` from `neovim`. 
You can also patch it, rename it, make it your own, 
the license allows you to do almost whatever you like with it [see license](./LICENSE).  

## Lazy.nvim example of default config, builds locally

```lua
return {
    -- Will pull the main branch and run `build.lua` 
    "MarcusGrass/nvim_winpick",
    ...
}
```

If the plugin spec is stated only as above, lazy will pull the main branch, find that it contains a `build.lua` 
[this file](./build.lua), and execute it. [See the lazy docs on plugin specs here](https://lazy.folke.io/spec).  

If examining the `build.lua`, you'll see that it tries to build the project using `cargo`, the `rust`-language build-tool.
The user needs that on their machine to be able to build it, assuming that it is, and a unix-like os is used, 
it should just work. If wanting to do it that way, install [rustup](https://doc.rust-lang.org/cargo/getting-started/installation.html), or source the `rust`-toolchain and `cargo` some other way.

The reason that this is the default will be explained by the prebuilt section.

## Prebuilt

The CI builds library-files (`.so`) that can be loaded without building, but these are compiled for each architecture.
Since your architecture is unknown (to me), none can be specified by default.  

```lua
return {
    "MarcusGrass/nvim_winpick",
    -- will pull a more 'bare' branch which doesn't contain much more than the library at `./lua/nvim_winpick.so`
    -- "aarch64-unknown-linux-gnu-latest" and "aarch64-apple-darwin" are also valid options
    branch = "x86_64-unknown-linux-gnu-latest",
    ...
}
```

The library is prebuilt for three targets (chosen because they're simple to build, theoretically it could also be 
prebuilt for windows, x86_64-mac, and `musl`-linux targets).  

### Potential issues

Prebuilding and distributing dynamically linked libraries is problematic. Mainly because of library dependencies, 

--- Output from `ldd` for a binary built on x86_64-unknown-linux-gnu
```
ldd target/lto/libnvim_winpick.so 
        linux-vdso.so.1 (0x00007ffbc2caf000)
        libgcc_s.so.1 => /usr/lib/gcc/x86_64-pc-linux-gnu/14/libgcc_s.so.1 (0x00007ffbc2bd7000)
        libc.so.6 => /usr/lib64/libc.so.6 (0x00007ffbc29eb000)
        /lib64/ld-linux-x86-64.so.2 (0x00007ffbc2cb1000)
```

This is built for gcc and has a dependency on `gcc` when loaded (as well as the `vdso` but that shouldn't be problematic on Linux) .
`gcc` respects forwards compatibility, i.e. a binary built with `gcc` version 13 should load properly on `gcc` version 14. 
For that reason, the GitHub runners use the oldest `Ubuntu` version available to build linux libraries.

#### Potential solution

If this plugin was built [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) this wouldn't be 
a problem, the `.so` would be without dependencies. That is 
indeed possible to do, but then [nvim-oxi](https://github.com/noib3/nvim-oxi) which contains the `neovim` bindings 
would have to be made `no_std`. I have forked it and done that, but it created quite the mess. A potential future burst of 
inspiration may lead me to do that properly, or create my own minimal bindings. 
But, for now I'm happy contributing to that project instead when I find issues.


## Build fails

The build can fail, both with the prebuilt and local-build options.

### Local

1. Is cargo installed?
2. Is there working access to the internet? `cargo` pulls dependencies.

### Prebuilt

1. Is the correct architecture chosen?
2. Are you running a linux-distribution with ancient packages (debian)?

