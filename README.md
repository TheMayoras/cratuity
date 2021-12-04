# cratuity

[![Crates.io](https://img.shields.io/crates/v/cratuity)](https://crates.io/crates/cratuity)
[![GitHub last commit](https://img.shields.io/github/last-commit/TheMayoras/cratuity)](https://github.com/TheMayoras/cratuity)

A TUI for quickly searching Crates.io

The searches done are return the same results as if you entered the search term
into the search bar on crates.io.  The results are returned in pages of 5
results each

![Cratuity demo](https://raw.githubusercontent.com/TheMayoras/cratuity/master/assets/demo.gif)

## Requirements and Optional Features

## Clipboard

To have access to the clipboard on _Linux_ `xorg-dev` must be installed

### To Turn Off Clipboard Access

`cargo install` must be used with the `no-copy` feature

```sh
cargo install cratuity --features no-copy
```

## Usage

When prompted for an input, press ESC to cancel the input or Enter to search
for what was entered.

When scrolling through history, press N to move a page down and P to move a
page up.  Press q to quit from the search screen.

When scrolling through the pages, you can change you search term at any time by
pressing f, or you can change the sorting method by pressing S.

## Copying

You can copy the Cargo.toml format for a crate's most recent version by
selecting the crate with J/K and then pressing C to copy the string to the
clipboard.  For example, selecting the `serde` crate and pressing C may cause
something like the following to be copied to your clipboard: `serde = "1.0.118"`

## Sorting

The 5 sorting methods are the exact same sorting methods that you can use to
search for crates on Crates.io.  These are

1. Relevancy
2. All Time Downloaded
3. Recently Downloaded
4. Recently Updated
5. Newly Added

_Please submit any issues for feature requests to the github repository!_
