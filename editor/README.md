# slide editor tools

## vim

[`slide.vim`](./slide.vim) contains a simple mapping that will replace a visual selection with the
output of slide on that selection. The mapping is invoked with `:slide` in visual mode. The mapping
requires a `slide` executable in your path.

After enabling the mapping, `Shift + V :slide` in [`slide.vim.example`](./slide.vim.example) should
replace the contents with `x + 2.5`.
