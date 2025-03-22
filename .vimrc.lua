local lspconfig = require 'lspconfig'
local highlight = vim.cmd.highlight

vim.cmd("colorscheme sunset_cloud")

highlight({ "Comment", "cterm=italic", "gui=italic" })

highlight({ "Normal", "ctermbg=NONE", "guibg=NONE" })
highlight({ "NormalNC", "ctermbg=NONE", "guibg=NONE" })

highlight({ "NonText", "ctermbg=NONE", "ctermfg=NONE" })
highlight({ "Visual", "ctermbg=NONE", "cterm=NONE" })

highlight({ "LineNr", "ctermbg=NONE" })
highlight({ "VertSplit", "ctermbg=NONE", "ctermfg=NONE" })

