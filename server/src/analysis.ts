
import { access, constants, readFile } from 'fs/promises'
import { inspect } from 'util'
import { InputData, ParseResult, parse_js } from './rust/does_it_throw_wasm'
import path = require('path')


const _checkAccessOnFile = async (file: string) => {
  try {
    await access(file, constants.R_OK)
    return Promise.resolve(file)
  } catch (e) {
    return Promise.reject(e)
  }
}

const findFirstFileThatExists = async (uri: string, relative_import: string) => {
  const isTs = uri.endsWith('.ts') || uri.endsWith('.tsx')
  const baseUri = `${path.resolve(path.dirname(uri.replace('file://', '')), relative_import)}`
  let files = Array(4)
  if (isTs) {
    files = [`${baseUri}.ts`, `${baseUri}.tsx`, `${baseUri}.js`, `${baseUri}.jsx`]
  } else {
    files = [`${baseUri}.js`, `${baseUri}.jsx`, `${baseUri}.ts`, `${baseUri}.tsx`]
  }
  return Promise.any(files.map(_checkAccessOnFile))
}

export const getAnalysisResults = async ({
  inputData,
  initialUri,
  errorLogCallback
}: {
  inputData: InputData
  initialUri: string
  errorLogCallback: (msg: string) => void
}) => {
  const analysis = parse_js(inputData) as ParseResult


  if (analysis.relative_imports.length > 0) {
    const filePromises = analysis.relative_imports.map(async (relative_import) => {
      try {
        const file = await findFirstFileThatExists(initialUri, relative_import)
        return {
          fileContent: await readFile(file, 'utf-8'), 
          fileUri: file
        }
      } catch (e) {
        errorLogCallback(`Error reading file ${inspect(e)}`)
        return undefined
      }
    })
    const files = (await Promise.all(filePromises)).filter((file) => !!file)
    const analysisArr = files.map((file) => {
      if (!file) {
        return undefined
      }
      const opts = {
        uri: file.fileUri,
        file_content: file.fileContent,
        ids_to_check: [],
        typescript_settings: {
          decorators: true
        }
      } satisfies InputData
      return parse_js(opts) as ParseResult
    })
    // TODO - this is a bit of a mess, but it works for now.
    // The original analysis is the one that has the throw statements Map()
    // We get the get the throw_ids from the imported analysis and then
    // check the original analysis for existing throw_ids.
    // This allows to to get the diagnostics from the imported analysis (one level deep for now)
    for (const import_analysis of analysisArr) {
      if (!import_analysis) {
        return
      }
      if (import_analysis.throw_ids.length) {
        for (const throw_id of import_analysis.throw_ids) {
          const newDiagnostics = analysis.imported_identifiers_diagnostics.get(throw_id)
          if (newDiagnostics?.diagnostics?.length) {
            analysis.diagnostics.push(...newDiagnostics.diagnostics)
          }
        }
      }
    }
  }
  return analysis
}
