# Slide Language Server

The slide language server (`slide_ls`) is a language server speaking the [LSP](https://microsoft.github.io/language-server-protocol/)
API, for use in interactive contexts like those of text editors.

`slide_ls` is the best way to work with slide in an context where you would like
real-time analysis and evaluation of math expressions. The language server is 
user-configurable to work with any kind of document that may have a math program
you would like to use with slide.

## Installation

Currently, the only way to install `slide_ls` is to build from source. `cargo
build` in this directory or in the [`slide` repo root](../../) will build a
`slide_ls` binary.

## Usage

To start using `slide_ls`, just add it as a language server available to your
editor's LSP client provider. For example, with [`coc.nvim`](https://github.com/neoclide/coc.nvim),
add `slide` as key to the `languageserver` option in your `coc-settings`:

```json
"languageserver": {
  "slide": {
    "command": "/path/to/slide_ls/binary",
    "rootPatterns": ["*"],
    "filetypes": ["md", "markdown", "math"],
    "initializationOptions": {
      "document_parsers": {
        "md": "```math\\n((?:.|\\n)*?)\\n```",
        "math": "((?:.|\\n)*)"
      }
    }
  }
}
```

### Initialization Options

#### `document_parsers`

For each filetype extension you would like `slide_ls` to work with, you should
provide a regex that will match "slide program blocks" in that file type.

For example, to tell `slide_ls` that content within <code>\`\`\`math</code>
language blocks in `*.md` files should be treated as slide programs, and all
content in `*.math` files should be treated as slide programs, specify the
following in your `initializationOptions`:

```json
"document_parsers": {
  "md": "```math\\n((?:.|\\n)*?)\\n```",
  "math": "((?:.|\\n)*)"
}
```

Now, slide_ls will answer language service queries in a file like `my_math_document.md`:

````markdown
# My math document

## Addition

slide_ls will answer queries in the following block:

```math
1 + 2
```

## Subtraction

slide_ls will report a diagnostic of a missing right operand in the following block:

```math
4 -
```
````

The "slide program block" regex provided for a filetype extension is subject to
the following constraints:

- The regex must contain exactly one explicit capturing group to denote the
  contents of a slide program. For example, `(.*)` and
  `` ```math\n((?:.|\n)*?)\n``` `` meet this requirement, while `.*`, `(.*)(.*)`,
  and `` ```math\n((.|\n)*?)\n``` `` do not.

- The regex will be parsed as a multi-line regex. Be sure to include newlines
  explicitly if you want them to be captured by the regex. For example, `(.*)`
  captures all characters except line feeds; to also capture line feeds, use
  `((?:.|\n)*)`.
