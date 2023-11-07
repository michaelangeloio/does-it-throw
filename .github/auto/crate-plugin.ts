import { Auto, execPromise, IPlugin, SEMVER } from '@auto-it/core'
import { readdir, readFile, writeFile } from 'fs/promises'
import { join, resolve } from 'path'
import { inc, ReleaseType } from 'semver'
import { parse as parseToml } from 'toml'
import { inspect } from 'util'
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
	console.log('\x1b[36m%s\x1b[0m', 'testing', toml)
  const members = await Promise.all(toml.workspace.members.map(async (member: string) => {
		console.log('\x1b[36m%s\x1b[0m', process.env.GITHUB_WORKSPACE, process.cwd())
		const files = await readdir(process.env.GITHUB_WORKSPACE || '');
		console.log('\x1b[36m%s\x1b[0m', process.env.GITHUB_WORKSPACE )
		files.forEach(file => {
			console.log(file, 'workspace');
		})
		const files2 = await readdir(process.cwd() || '');
		console.log('\x1b[36m%s\x1b[0m', process.cwd())
		files2.forEach(file => {
			console.log(file, 'cwd');
		})
		const anotherPath = resolve(process.cwd() || process.env.GITHUB_WORKSPACE || '', member)
		console.log('\x1b[36m%s\x1b[0m', anotherPath)
		const files3 = await readdir(anotherPath);
		files3.forEach(file => {
			console.log(file, 'w/ member');
		})
		const trimmedDir =  anotherPath.replace('/does-it-throw', '')
		console.log('\x1b[36m%s\x1b[0m', trimmedDir)
		const files4 = await readdir(trimmedDir);
		files4.forEach(file => {
			console.log(file, 'w/ member trimmed');
		})

    const packagePath = resolve(process.cwd() || process.env.GITHUB_WORKSPACE || '', member, 'Cargo.toml')
		console.log('\x1b[36m%s\x1b[0m', packagePath)
    const packageContent = (await readFile(packagePath)).toString()
    const packageToml = parseToml(packageContent.toString())
    return {
      packagePath,
      packageToml,
      packageContent,
    }
  }))
	console.log('\x1b[36m%s\x1b[0m', inspect(members))
	return members
}

/** Bump the version in cargo.toml */
export async function bumpVersions(bumpBy: SEMVER) {
  const workspaceMembers = await getWorkspaceMembers()
  const promises = workspaceMembers.map(async (member) => {
    const { packagePath, packageContent, packageToml } = member
		console.log('\x1b[36m%s\x1b[0m', packageContent, packagePath, packageToml)
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
