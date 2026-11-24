# Contributing

Bugs and ideas go to the [issues](https://github.com/Ziuziuk-Denys/Flexolotl/issues). For anything larger than a fix, open an issue first so the work is not wasted.

## Before a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## How the code is written

- One file does one thing. `crates/core` knows nothing about the interface.
- Names instead of comments. A comment is for what the code cannot say.
- A new dependency needs a reason in the pull request.
- What differs between systems lives in `crates/app/src/platform` and `crates/app/src/music/system`.
- Every text exists in English and Russian, in `crates/app/src/copy`.

## Rules the app keeps

- Only the official local connection to Discord. No user tokens, no client mods.
- Nothing leaves the computer without the interface saying so.
