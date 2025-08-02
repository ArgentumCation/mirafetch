# mirafetch

---

![GitHub](https://img.shields.io/github/license/argentumcation/mirafetch?color=blue)

<!--![GitHub release (with filter)](https://img.shields.io/github/v/release/argentumcation/mirafetch)
![docs.rs](https://img.shields.io/docsrs/mirafetch)
![Crates.io](https://img.shields.io/crates/d/mirafetch)
![Repology - Repositories](https://img.shields.io/repology/repositories/mirafetch)-->

A Rust reimplementation of Hyfetch wih a focus on speed

## Installation

### Homebrew

```
brew tap ArgentumCation/mirafetch https://github.com/ArgentumCation/mirafetch
brew install [--head] mirafetch
```

### Manual

Download the repo and run `cargo run` in the folder to try it out. To install mirafetch try `cargo install .` and ensure your cargo directory is in `$PATH`

## Images
Images:
![image](https://github.com/user-attachments/assets/8c2ce3cd-4870-4441-94e3-9d2469f0dcd7)
![image](https://github.com/user-attachments/assets/9674b066-f736-408c-af2d-6a62fa2db89b)
![image](https://github.com/user-attachments/assets/f2c5abb6-0f93-4782-836b-9f88e9385a4e)


## Configuration

### CLI Config

```
Options:
-s, --scheme-name <SCHEME_NAME>
-o, --orientation <ORIENTATION> [possible values: horizontal, vertical]
-i, --icon-name <ICON_NAME>
-h, --help Print help
-V, --version Print version
```

### Config file

- The configuration file is located in:

  - Linux: `TODO/config.toml`
  - macOS: `TODO/config.toml`
  - Windows `TODO\config.toml`

- `icon_name` is optional and overrides the default icon for your system, these are defined in `data/data.yaml`
- `scheme_name` is optional and defines the flag pattern to overlay on your OS icon, these are defined in `data/flags.toml`
  - `orientation` is required when `scheme_name` is present, and can be `Horizontal` or `Vertical`, and sets the direction of the flag's stripes

## Notes

- I could definitely use help testing on other platforms
- There's currently minimal support for macOS, if you have a Mac and want to port this, feel free to make a PR
- Documentation is also sparse at the moment, feel free to create an issue or open a PR
- We're currently targeting a run time of ~60ms (approximately 15 fps), if you have any optimizations, please let us know!

## Tested on:

- Windows 11
- Ubuntu 23.10
- Pop! OS 20.04
- NixOS 24.10
- macOS Sonoma (14)
- macOS Ventura (13)

## Special Thanks

- https://github.com/fastfetch-cli/fastfetch
- https://github.com/hykilpikonna/hyfetch/
