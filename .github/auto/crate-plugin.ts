import { Auto, execPromise, IPlugin, SEMVER } from '@auto-it/core'
import { readFile, writeFile } from 'fs/promises'
import { join } from 'path'
import { inc, ReleaseType } from 'semver'
import { parse as parseToml } from 'toml'
/** Get the parsed cargo.toml for the crate */
export const getCargoConfig = async () => {
  const content = (await readFile(join(process.cwd() || process.env.GITHUB_WORKSPACE || '', 'Cargo.toml'))).toString()
  return { toml: parseToml(content), content }
}

/** Get the credentials for publishing to crates.io */
export async function checkForCreds() {
  return process.env.CARGO_REGISTRY_TOKEN
}

export async function getWorkspaceMembers(): Promise<
  {
    packagePath: string
    packageToml: any
    packageContent: string
  }[]
> {
  const { toml } = await getCargoConfig()
  return toml.workspace.members.map(async (member: string) => {
    const packagePath = join(process.cwd() || process.env.GITHUB_WORKSPACE || '', member, 'Cargo.toml')
    const packageContent = (await readFile(packagePath)).toString()
    const packageToml = parseToml(packageContent.toString())
    return {
      packagePath,
      packageToml,
      packageContent,
    }
  })
}

/** Bump the version in cargo.toml */
export async function bumpVersions(bumpBy: SEMVER) {
  const workspaceMembers = await getWorkspaceMembers()
  const promises = workspaceMembers.map(async (member) => {
    const { packagePath, packageContent, packageToml } = member
    const versionOld = packageToml.package.version
    const versionNew = inc(versionOld, bumpBy as ReleaseType)

    if (!versionNew) {
      throw new Error(`Could not increment previous version: ${versionOld}`)
    }

    const replaceOld = `version = "${versionOld}"`
    const replaceNew = `version = "${versionNew}"`
    const contentNew = packageContent.replace(replaceOld, replaceNew)
    await writeFile(packagePath, contentNew)
    return versionNew
  })
  return await Promise.all(promises)
}

/** Deploy Rust crates */
export default class CratesPlugin implements IPlugin {
  /** The name of the plugin */
  name = 'crates-plugin'

  /** Tap into auto plugin points. */
  apply(auto: Auto) {
    auto.hooks.beforeShipIt.tap(this.name, () => {
      if (!checkForCreds()) {
        throw new Error('Cargo token is needed for the Crates plugin!')
      }
    })

    auto.hooks.getAuthor.tapPromise(this.name, async () => {
      const { toml } = await getCargoConfig()
      const authors = toml.workspace.authors
      auto.logger.log.info(`Crate authors: ${authors}`)
      return authors
    })

    auto.hooks.getPreviousVersion.tapPromise(this.name, async () => {
      const { toml } = await getCargoConfig()
      const version = auto.prefixRelease(toml.workspace.version)
      auto.logger.log.info(`Crate version: ${version}`)
      return version
    })

    auto.hooks.version.tapPromise(this.name, async ({ bump, dryRun, quiet }) => {
      const newVersions = await bumpVersions(bump)
      newVersions.forEach(async (versionNew) => {
        if (dryRun) {
          if (quiet) {
            console.log(versionNew)
          } else {
            auto.logger.log.info(`Would have published: ${versionNew}`)
          }

          return
        }

        auto.logger.log.info(`Bumped version to: ${versionNew}`)
      })
      auto.logger.log.info('Building to update Cargo.lock')
      await execPromise('cargo', ['build'])
      auto.logger.verbose.info('Committing files')
      await execPromise('git', ['add', 'Cargo.toml', 'Cargo.lock'])
      await execPromise('git', ['commit', '-m', `'Bump version to: ${newVersions[0]} [skip ci]'`, '--no-verify'])
      auto.logger.log.info('Create git commit for new version')
    })

    auto.hooks.publish.tapPromise(this.name, async () => {
      const workspaceMembers = await getWorkspaceMembers()
      const promises = workspaceMembers.map(async (member) => {
        const { packageToml } = member
        auto.logger.log.info('Publishing via cargo')
        await execPromise('cargo', ['publish', packageToml.name])
        auto.logger.log.info('Publish complete')
        auto.logger.log.info('Pushing local git changes to origin remote')
        await execPromise('git', ['push', '--follow-tags', '--set-upstream', auto.remote, auto.baseBranch])
        auto.logger.log.info('Push complete')
      })
      await Promise.all(promises)
    })
  }
}
