/* eslint-disable @typescript-eslint/no-explicit-any */
import { watch } from 'chokidar'
import { build as esbuild } from 'esbuild'
import copy from 'esbuild-plugin-copy'

const log = (...args: any[]) => console.log('\x1b[36m%s\x1b[0m', '[watcher]', ...args)
const logError = (...args: any[]) => console.log('\x1b[31m%s\x1b[0m', ...args)

const compile = async ({ path, ready, init }: { path: string; ready: boolean; init: boolean }) => {
  if (!ready) {
    return
  }
  const plugins: any = []
  if (path.includes('.rs') || init) {
    plugins.push(
      copy({
        resolveFrom: 'cwd',
        assets: {
          from: ['./server/src/rust/**/*'],
          to: ['./server/out/rust'],
        },
        watch: true,
      }) as any,
    )
    // TODO - use bun to bundle the code when it supports CJS (or vscode supports ESM)
    const command = 'wasm-pack build crates/does-it-throw-wasm --target nodejs --out-dir server/src/rust'
    try {
      log('building wasm')
      const result = Bun.spawnSync(command.split(' '), {
        cwd: import.meta.dir,
        onExit: (proc, exitCode, signalCode, error) => {
          logError(proc, exitCode, signalCode, error)
          return
        },
      })

      log(result.stderr.toString())
      log(result.stdout.toString())
    } catch (e) {
      log(e)
    }
  }

  const compileClient = async () => {
    log('compiling client ts')
    await esbuild({
      outdir: 'client/out',
      entryPoints: ['client/src/extension.ts'],
      format: 'cjs',
      tsconfig: 'client/tsconfig.json',
      plugins,
    })
    log('done compiling client')
  }

  const compileServer = async () => {
    log('compiling server ts')
    await esbuild({
      outdir: 'server/out',
      entryPoints: ['server/src/server.ts'],
      format: 'cjs',
      tsconfig: 'server/tsconfig.json',
      plugins,
    })
    log('done compiling server')
  }

  await Promise.all([compileClient(), compileServer()])
}

async function main() {
  const isWatch = process.execArgv.find((arg) => arg.includes('watch'))
  if (!isWatch) {
    log('not in watch mode, compiling once')
    await compile({
      init: true,
      ready: true,
      path: '',
    })
    return
  }

  const watcher = watch('**/*.{ts,rs}', {
    ignored: '(node_modules|target)/**/*',
  })
  let ready = false

  watcher.on('ready', async () => {
    log('ready')
    ready = true
    await compile({
      init: true,
      ready,
      path: '',
    })
  })

  watcher.on('change', async (path) => {
    await compile({
      init: false,
      path,
      ready,
    })
  })
  watcher.on('add', async (path) => {
    await compile({
      path,
      ready,
      init: false,
    })
  })
}

main()
