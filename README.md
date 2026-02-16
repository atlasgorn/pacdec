# Pacdec - A simple declarative package manager for Arch Linux

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/atlasgorn/pacdec#license)

## About

`Pacdec` (pacman declare) is a wrapper for `pacman` or `AUR` helpers that uses [kdl](https://kdl.dev) files to declare packages in a simple way.

Its main focus is to provide control through declarative management while maintaining the ease of use of default `pacman`.

`Pacdec` makes extensive use of categories (often referred as cat) and tags for package management and rich features such: `hooks`, `packages external config files` and `per-device config`. Categories add implicit tag to packages and can be nested.

## Motivation

Most declarative package managers either have overly verbose syntax or have only very basic functionality. `Pacdec` have simple and consistent syntax yet offers rich features.

Another problem with most declarative package managers is how you often lose the simplicity of running `sudo pacman -Syu package` and have to only manually edit declaration file and then sync, `pacdec` offers simple `pacdec ins package`.

## Installation

Currently the only way to install `pacdec` is from source. Later on `AUR` and `cargo` packages will be added.

### Building from source

```bash
git clone https://github.com/atlasgorn/pacdec
cd pacdec
cargo build --release
cp target/release/pacdec /usr/local/bin
```

### Prerequisites

- rust
- paru/yay/pacman
- fzf

## Quick start

Run `pacdec gen[ereate]` to generate `packages.kdl` file at default location (`~/.config/pacdec/`) with all **explicitly** installed packages sorted by time (with the use of `pacman log`) placed in default category (`cat:uncat`).

Edit declaration file. Organize packages by files and categories and use `pacdec ins`/`pacdec unins` instead of `pacman -Sy`/`pacman -Rns`. Or manually edit declaration file and then run `pacdec sync` to fully **sync** the system state to declaration file.

It probably makes sense to separate packages installed by your distro into different file and link it with `@include path/to/file.kdl` (could be simply done by cutting topmost packages).

## Usage

`pacdec add|ins[tall] pkg1 pkg2 --cat=catname/subcatname` to **add** or **add and install** `pkg1`, `pkg2` to category `catname/subcatname`.

`pacdec remove(or rm)|unins[tall] pkg1 pkg2` to **remove** or **remove and uninstall** `pkg1`, `pkg2`.

If **packages** and/or **category** is not specified you will be prompted with `fzf` to select them.

### Minimal example of declaration file

```kdl
cat:category_name {
    package1 tag1
    package2 tag1 tag2 {
        subpackage3 tag1
    }
}

cat:system {
    linux
    base
    base-devel
    linux-firmware
}

cat:dev {
    neovim terminal
    python {
        uv // it makes no sense to have uv without python so we add it as subpackage
    }
}

```

## Declaration file syntax

```kdl
@include default_distro_packages.kdl // includes specified file, useful for organizing packages into multiple files
@file cat:filewise_category // syntaxic sugar that replaces cat:filewise_category { /* content of the file */ }

cat:catname {
    package1 tag1 tag2
    "repos/package2" tag1 // kdl needs quotes for package names with slashes
    package3 {
        package4
        :options {
            type "full"
        }

    }
    package5 tag3 {
        hook:after_sync "sudo systemctl enable --now package5.service" // after_sync hook will run after syncing package5
        :options
    }
    cat:subcategory{
        package6 tag2
    }

}
```

In this example `package1` will have explicit tags `tag1` and `tag2` and implicit tag `catname` and `filewise_category`, `package6` will have explicit tag `tag2` and implicit tags `catname`, `subcategory` and `filewise_category`. Full `subcategory` path is `filewise_category/catname/subcategory`.

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
