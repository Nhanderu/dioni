# Dioni

[![License][badge-1-img]][badge-1-link]
[![Crates.io][badge-2-img]][badge-2-link]

A CLI that shuffle plays your saved tracks on Spotify.

## Why

I may be too paranoid, but I always felt like Spotify's shuffle play is fake. It
seems to play the same bands, or at least have some kind of intelligence behind
it.

So this project was born with the intention of having a **true random** queue
with your saved tracks.

But, again, I may be just too paranoid.

## Things you need to know

- It asks for your authentication opening your browser.
- It caches the authentication token so it doesn't authenticate again. You can
force authentication again with the flag `--force-auth`.
- The cache path is defined by `$DIONI_CACHE`. If not defined, it tries
`$XDG_CACHE_HOME/dioni`. If it's also not defined, it gets the default cache dir
for you OS (Linux: `~/.cache`, Mac: `~/Library/Caches`, Windows:
`~\AppData\Local`) plus `/dioni`. You can check it via `--cache-path` flag.
- If the directory in the cache path doesn't exists, it'll be created.
- If your total liked tracks exceeds our limit, it's gonna ask if you want to
add them in the queue. This can be pre-defined with the flags `--ignore-excess`
and `--add-excess-to-queue`.

## Install

### Brew

```sh
brew tap Nhanderu/packages
brew install dioni
```

### Cargo

```sh
cargo install dioni
```

## Run

#### `dioni`

Runs the program with default configuration.

### Flags

#### `-h` or `--help`

Shows the CLI help message.

#### `-v` or `--version`

Shows the CLI version.

#### `-q` or `--quiet`

Runs the program without writing to stdout. Requires `--add-excess-to-queue` or
`--ignore-excess`.

#### `--ignore-excess`

Ignore songs that exceed the Spotify limit.

#### `--add-excess-to-queue`

Add songs that exceed the Spotify limit to the queue.

#### `-a` or `--force-auth`

Clears the authentication cache, forcing it to ask for authentication again.

#### `--cache-path`

Shows the cache path.

## License

This project code is in the public domain. See the [LICENSE file][1].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be in the public domain, without any
additional terms or conditions.

[1]: ./LICENSE

[badge-1-img]: https://img.shields.io/github/license/Nhanderu/dioni?style=flat-square
[badge-1-link]: https://github.com/Nhanderu/dioni/blob/master/LICENSE
[badge-2-img]: https://img.shields.io/crates/v/dioni?style=flat-square
[badge-2-link]: https://crates.io/crates/dioni
