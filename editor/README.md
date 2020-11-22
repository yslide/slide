# slide editor tools

## language server

[`slide_ls`](./language_server) is an LSP implementation for slide. The language
server works is user-configurable to work with a variety of documents, and is
the best way to use slide's analysis tools in an interactive setting.

For more information, see the [server README](./language_server/README.md).

## vim

The following mapping will replace a visual selection with the output of slide
on that selection. The mapping is invoked with `:slide` in visual mode. The
mapping requires a `slide` executable in your path.

```vim
vnoremap :slide<C-R> :'<,'>!slide "$(cat)"<C-R>
```

After enabling the mapping, visual selection and execution of `:slide` on

```math
1 / 2 * 5 + x
```

[`slide.vim.example`](./slide.vim.example) should replace the contents with
`x + 2.5`.
