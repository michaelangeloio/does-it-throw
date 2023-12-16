> *There are probably ways to improve the Neovim experience. If you have suggestions, please open an issue!*

# Install the LSP Server 

First, install (globally) the `does-it-throw-lsp` package from NPM:
```bash
npm i -g does-it-throw-lsp
```

> You can use your favorite package manager, eg `bun install`

This package contains the same LSP Server VSCode runs under the hood. The server itself uses Node.js, but the core of the code is written in Rust and compiled to WASM. The server is published to NPM, and the Rust code is published to [crates.io](https://crates.io/crates/does-it-throw).

> If anyone has ideas on how to run the server via bun with some shim, let me know! The current `bin` script is located [here](https://github.com/michaelangeloio/does-it-throw/blob/main/server/bin/does-it-throw).

# Lua Setup

(optional) install Lazy.nvim:
```lua
-- init.lua
local lazypath = vim.fn.stdpath('data') .. '/lazy/lazy.nvim'
local uv = vim.uv or vim.loop

-- Auto-install lazy.nvim if not present
if not uv.fs_stat(lazypath) then
    print('Installing lazy.nvim....')
    vim.fn.system(
        {'git', 'clone', '--filter=blob:none', 'https://github.com/folke/lazy.nvim.git', '--branch=stable', -- latest stable release
         lazypath})
    print('Done.')
end

vim.opt.rtp:prepend(lazypath)
```
## Starting the Server

You can start the server manually by running the following command:
```bash
does-it-throw-lsp --stdio
```

### Lua Setup (cont'd)

Tell Neovim to use the LSP server:
```lua
require('lazy').setup({{'neovim/nvim-lspconfig'}})

local lsp_configurations = require('lspconfig.configs')

local server_config = {
    ["doesItThrow"] = {
        throwStatementSeverity = "Hint",
        functionThrowSeverity = "Hint",
        callToThrowSeverity = "Hint",
        callToImportedThrowSeverity = "Hint",
        maxNumberOfProblems = 10000
    }
}

-- Setup doesItThrow
if not lsp_configurations.does_it_throw_server then
    lsp_configurations.does_it_throw_server = {
        default_config = {
            cmd = {"does-it-throw-lsp", "--stdio"},
            filetypes = {"typescript", "javascript"},
            root_dir = function(fname)
                return vim.fn.getcwd()
            end
        }
    }
end

require'lspconfig'.does_it_throw_server.setup {
    on_init = function(client)
        client.config.settings = server_config
    end,
		-- important to set this up so that the server can find your desired settings
    handlers = {
        ["workspace/configuration"] = function(_, params, _, _)
            local configurations = {}
            for _, item in ipairs(params.items) do
                if item.section then
                    table.insert(configurations, server_config[item.section])
                end
            end
            return configurations
        end
    }
}
```
## Customizations
Notice the above lua config:
```lua
["doesItThrow"] = {
    throwStatementSeverity = "Hint",
    functionThrowSeverity = "Hint",
    callToThrowSeverity = "Hint",
    callToImportedThrowSeverity = "Hint",
    maxNumberOfProblems = 10000
}
```
The settings correspond to the same VSCode settings. These settings and descriptions can be found under [package.json](https://github.com/michaelangeloio/does-it-throw/blob/main/package.json).



### (optional) Customize your diagnostics:
```lua
vim.lsp.handlers["textDocument/publishDiagnostics"] = vim.lsp.with(vim.lsp.diagnostic.on_publish_diagnostics, {
    -- Enable underline, use default values
    underline = true,
    -- Enable virtual text, override spacing to 4
    virtual_text = {
        spacing = 4,
        prefix = '●' -- This can be any character you prefer
    },
    -- Use a function to define signs
    signs = true,
    -- Disable a feature
    update_in_insert = false
})
local signs = {
    Error = " ",
    Warn = " ",
    Hint = " ",
    Info = " "
}
for type, icon in pairs(signs) do
    local hl = "LspDiagnosticsSign" .. type
    vim.fn.sign_define(hl, {
        text = icon,
        texthl = hl,
        numhl = hl
    })
end
```

If you need to debug the server for whatever reason, you can set the log level to `debug`:
```lua
-- Enable server log diagnostics if you want
vim.lsp.set_log_level("debug")
```