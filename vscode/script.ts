import { createVSIX } from '@vscode/vsce'
import path from 'path'



function main() {
  createVSIX({

    ignoreFile: undefined,
    useYarn: true,
    dependencies: true,
    dependencyEntryPoints: [`${path.join(__dirname)}/out/does_it_throw_wasm_bg.wasm`]
  })
}

main()
