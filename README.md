# wandering_inn_scraper

## Description

This is a scraper for the web serial [The Wandering Inn](https://wanderinginn.com/).
This is written to generate most of the variations of epubs that would be useful to you.

## Usage

1. Build the latest version, or download [the latest release](https://github.com/rsauvehoover/wandering_inn_scraper/releases)
2. add a `config.json` to the same directory as the binary or the root of the project if building from source.
   See [example_config.json](example_config.json) for an example of config options.
3. Run the program, outputs will be in the `build` directory.
   NOTE: While you can run run the program by double clicking the binary, it will close immediately after finishing
   and you won't be able to see any output. It is recommended to run from a terminal.

## Building/running locally

1. Ensure you have rust installed, if not install [here](https://www.rust-lang.org/tools/install).
2. Clone this repo.

```bash
git clone https://github.com/rsauvehoover/wandering_inn_scraper.git
```

3. Build the project with cargo. `--release` flag is optional if you don't want optimizations.
   This step can be skipped if you want, `cargo run` will also build if necessary

```bash
cargo build --release
```

4. Run the project with cargo. `--release` flag is optional if you don't want optimizations

```bash
cargo run --release
```

## Build

Binaries will be found `target/release/bundle` and `target/wix` directories

### Linux/MacOS

```bash
cargo bundle --release
```

### Windows

NOTE: `cargo wix` doesn't show any output by default, run with `-v` and `--nocapture` flags to see verbose output.

```bash
cargo wix
```

## Versioning

```bash
cargo bump {major|minor|patch} --git-tag
```
