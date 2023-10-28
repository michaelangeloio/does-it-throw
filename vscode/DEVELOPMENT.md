# Development

## Language Server Protocol (LSP) Functionality

### Requirements

- Latest stable version of Rust with `cargo` available on your system PATH.
- [`wasm-pack`][wasm-pack] 0.9.1+ installed and available on your system PATH.
- VS Code 1.52.0+.

### Steps to test the extension out in VS Code

1. Run `make package` in the current directory (where this file lives).
2. Install the output `.vsix` file into your local VS Code instance: `code --install-extension does-it-throw.vsix`.
3. Run the **Developer: Reload Window** (`workbench.action.reloadWindow`)
   command.
