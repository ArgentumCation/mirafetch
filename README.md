# mirafetch
-------
![GitHub](https://img.shields.io/github/license/argentumcation/mirafetch?color=blue)
<!--![GitHub release (with filter)](https://img.shields.io/github/v/release/argentumcation/mirafetch)
![docs.rs](https://img.shields.io/docsrs/mirafetch)
![Crates.io](https://img.shields.io/crates/d/mirafetch)
![Repology - Repositories](https://img.shields.io/repology/repositories/mirafetch)-->

A Rust reimplementation of Hyfetch wih a focus on speed

## Installation
Download the repo and run `cargo run` in the folder to try it out. To install mirafetch try `cargo install .` and ensure your cargo directory is in `$PATH`

## Configuration
- The configuration file is located in:
  - Linux: `TODO/config.toml`
  - macOS: `TODO/config.toml`
  - Windows `TODO\config.toml`

- `icon_name` is optional and overrides the default icon for your system, these are defined in `data/data.yaml`
- `scheme_name` is optional and defines the flag pattern to overlay on your OS icon, these are defined in `data/flags.toml`
  - `orientation` is required when `scheme_name` is present, and can be `Horizontal` or `Vertical`, and sets the direction of the flag's stripes
## Notes
- I could definitely use help testing on other platforms
- There's currently no support for macOS, if you have a Mac and want to port this, feel free to make a PR

## Tested on:
- Windows 11
- Ubuntu 23.10
- Pop! OS 20.04

## Special Thanks
- https://github.com/fastfetch-cli/fastfetch
- https://github.com/hykilpikonna/hyfetch/
