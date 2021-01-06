# cratuity

A TUI for quickly searching Crates.io

The searches done are return the same results as if you entered the search term
into the search bar on crates.io.  The results are returned in pages of 5
results each

![Cratuity demo](https://raw.githubusercontent.com/TheMayoras/cratuity/master/assets/demo.gif)

## Usage

When prompted for an input, press ESC to cancel the input or Enter to search
for what was entered.

When scrolling through history, press J to move a page down and K to move a
page up.  Press q to quit from the search screen.

When scrolling through the pages, you can change you search term at any time by
pressing f, or you can change the sorting method by pressing S.

## Sorting

The 5 sorting methods are the exact same sorting methods that you can use to
search for crates on Crates.io.  These are

1. Relevancy
2. All Time Downloaded
3. Recently Downloaded
4. Recently Updated
5. Newly Added
