> *NeoVim is hacky right now, but it works!*
# Lua
Here's an example init.lua config:
```lua
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
            cmd = {"node", "/Users/angelo/developer/does-it-throw-vscode-0.2.5/extension/server/out/server.js",
                   "--stdio"},
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
		-- important to set this up so that the server can find the settings
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

-- Enable diagnostics if you want
vim.lsp.set_log_level("debug")
```