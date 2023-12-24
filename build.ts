import { watch } from 'chokidar'
import { build as esbuild } from 'esbuild'
import copy from 'esbuild-plugin-copy'
import { Errorlike, readableStreamToText } from 'bun'

const log = (...args: any[]) => console.log('\x1b[36m%s\x1b[0m', '[builder]', ...args)
const logError = (...args: any[]) => console.log('\x1b[31m%s\x1b[0m', '[builder]', ...args)

const isError = (exitCode: number) => {
  return exitCode !== 0
}

const compile = async ({
  path,
  ready,
  init
}: {
  path: string
  ready: boolean
  init: boolean
}) => {
  if (!ready) {
    return
  }
  const plugins: any = []
  const errors = []

  /**
   * Compile WASM if the file is a rust file or if this is the first time
   */

  const compileWasm = async () => {
    // TODO - use bun to bundle the code when it supports CJS (or vscode supports ESM)
    const command = 'wasm-pack build crates/does-it-throw-wasm --target nodejs --out-dir ../../server/src/rust'
    log('building wasm')
    const { stdout, exited } = Bun.spawn(command.split(' '), {
      cwd: __dirname
    })
    const [stdOut, exitCode] = await Promise.all([await readableStreamToText(stdout), exited])
    if (isError(exitCode)) {
      return {
        isError: true,
        error: stdOut,
        type: 'wasm'
      }
    }
    log('done building wasm', stdOut)
    return {
      isError: false,
      error: null,
      type: 'wasm'
    }
  }

  const typeCheck = async (project: 'server' | 'client') => {
    const command = `tsc -p ${project}/tsconfig.json --noEmit`
    log('type checking')
    const { exited, stdout } = Bun.spawn(command.split(' '), {
      cwd: __dirname
    })
    const [stdOut, exitCode] = await Promise.all([await readableStreamToText(stdout), exited])
    if (isError(exitCode)) {
      return {
        isError: true,
        error: stdOut,
        type: `type-check-${project}`
      }
    }
    log('done type checking', stdOut)
    return {
      isError: false,
      error: null,
      type: `type-check-${project}`
    }
  }

  /**
   * Compile/Bundle the client
   */
  if (path.includes('.rs') || init) {
    plugins.push(
      copy({
        resolveFrom: 'cwd',
        assets: {
          from: ['./server/src/rust/**/*'],
          to: ['./server/out']
        },
        watch: true
      }) as any
    )
  }
  const compileClient = async () => {
    try {
      log('compiling client ts')
      await esbuild({
        minify: true,
        sourcemap: true,
        bundle: true,
        outdir: 'client/out',
        entryPoints: ['client/src/extension.ts'],
        external: ['vscode'],
        platform: 'node',
        format: 'cjs',
        tsconfig: 'client/tsconfig.json',
        plugins
      })
      log('done compiling client')
      return {
        isError: false,
        error: null,
        type: 'client-compile'
      }
    } catch (error) {
      return {
        isError: true,
        error,
        type: 'client-compile'
      }
    }
  }

  /**
   * Compile/Bundle the server
   */
  const compileServer = async () => {
    try {
      log('compiling server ts')
      await esbuild({
        outdir: 'server/out',
        minify: true,
        bundle: true,
        sourcemap: true,
        platform: 'node',
        external: ['vscode'],
        entryPoints: ['server/src/server.ts'],
        format: 'cjs',
        tsconfig: 'server/tsconfig.json',
        plugins
      })
      log('done compiling server')
      return {
        isError: false,
        error: null,
        type: 'server-compile'
      }
    } catch (error) {
      return {
        isError: true,
        error,
        type: 'server-compile'
      }
    }
  }

  const copyREADME = async () => {
    const [readMe, license] = await Promise.all([
      await Bun.file('README.md').text(),
      await Bun.file('LICENSE.txt').text()
    ])
    const baseUrl = 'https://github.com/michaelangeloio/does-it-throw/blob/main'
    function replaceLinks(markdown: string, prefix: string) {
      const regex = /!\[.*?\]\((\.\/assets\/(basic-throw|calltothrow).*?)\)/g
      return markdown.replace(regex, (match, p1) => {
        return match.replace(p1, prefix + p1.substring(1))
      })
    }
    const newReadMe = replaceLinks(readMe, baseUrl)
    log('writing new readme')
    await Promise.all([Bun.write('server/README.md', newReadMe), Bun.write('server/LICENSE.txt', license)])
    return {
      isError: false,
      error: null,
      type: 'copy-readme'
    }
  }

  /**
   * Build everything in parallel after wasm
   */
  if (path.includes('.rs') || init) {
    const result = await compileWasm()
    if (result.isError) {
      errors.push(result)
      return errors
    }
  }

  errors.push(
    ...(await Promise.all([compileClient(), compileServer(), typeCheck('server'), typeCheck('client'), copyREADME()]))
  )
  return errors
}

async function main() {
  const isWatch = process.argv.includes('--watch')
  if (!isWatch) {
    log('not in watch mode, compiling once')
    const errors = await compile({
      init: true,
      ready: true,
      path: ''
    })
    const hasErrors = errors?.filter((error) => error.isError)
    if (hasErrors?.length) {
      logError('errors compiling')
      for (const error of hasErrors) {
        logError(error)
      }
      throw new Error('exiting')
    }
    return
  }

  await compile({
    init: true,
    ready: true,
    path: ''
  })

  const watcher = watch('**/*.{ts,rs}', {
    ignored: ['(node_modules|target)/**/*', 'server/out/**/*', 'client/out/**/*']
  })
  let ready = false

  watcher.on('ready', async () => {
    log('ready')
    ready = true
  })

  watcher.on('change', async (path) => {
    log('change', path)
    await compile({
      init: false,
      path,
      ready
    })
  })
  watcher.on('add', async (path) => {
    await compile({
      path,
      ready,
      init: false
    })
  })
}

main()
