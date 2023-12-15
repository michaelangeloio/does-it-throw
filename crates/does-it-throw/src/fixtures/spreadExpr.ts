// @ts-nocheck
// should work for private class
class SomeClass {
  constructor(public x: number) {}

  async _contextFromWorkflow() {
    throw new Error('Some error')
  }

  async someCallToThrow() {
    const { user, stravaUser, streakContext } = opts?.contextFromWorkFlow ?? (await this._contextFromWorkflow(job))
  }
}

// should work for exported class
// biome-ignore lint/suspicious/noRedeclare: <explanation>
export class SomeClass {
  constructor(public x: number) {}

  async _contextFromWorkflow() {
    throw new Error('Some error')
  }

  async someCallToThrow() {
    const { user, stravaUser, streakContext } = opts?.contextFromWorkFlow ?? (await this._contextFromWorkflow(job))
  }
}
