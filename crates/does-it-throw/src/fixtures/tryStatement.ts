// @ts-nocheck
export const someConstThatThrows = () => {
  try {
    throw new Error('never gonna give you up')
  } catch (e) {
    console.log(e)
  }
}

function callToConstThatThrows4() {
  someConstThatThrows()
}

const someCondition = true
export class Something {
  constructor() {
    try {
      throw new Error('hi khue')
    } catch (e) {
      console.log(e)
    }
  }

  someMethodThatThrows() {
    try {
      throw new Error('hi khue')
    } catch (e) {
      console.log(e)
    }
  }

  someMethodThatDoesNotThrow() {
    console.log('hi khue')
  }

  someMethodThatThrows2() {
    if (someCondition) {
      throw new Error('hi khue')
    }
  }

  nestedThrow() {
    try {
      if (someCondition) {
        return true
      }
      throw new Error('hi khue')
    } catch (e) {
      console.log(e)
    }
  }

  callNestedThrow() {
    if (someCondition) {
      return true
    }
    if (someCondition) {
      return true
    }
    this.nestedThrow()
  }
}

const _somethingCall = () => {
  const something = new Something()
  something.someMethodThatThrows()
}

export const somethingCall = () => {
  const something = new Something()
  something.someMethodThatThrows()
}

function _somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}

export function somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}

// @ts-nocheck
// should work for private class
class SomeClass {
  constructor(public x: number) {}

  async _contextFromWorkflow() {
    try {
      throw new Error('Some error')
    } catch (e) {
      console.log(e)
    }
  }

  async someCallToThrow() {
    const { user, stravaUser, streakContext } = opts?.contextFromWorkFlow ?? (await this._contextFromWorkflow(job))
  }
}

// should work for exported class
// biome-ignore lint/suspicious/noRedeclare: <explanation>
export class SomeClass2 {
  constructor(public x: number) {}

  async _contextFromWorkflow() {
    try {
      throw new Error('Some error')
    } catch (e) {
      console.log(e)
    }
  }

  async someCallToThrow() {
    const { user, stravaUser, streakContext } = opts?.contextFromWorkFlow ?? (await this._contextFromWorkflow(job))
  }
}

const server = http.createServer(async (req, res) => {
  switch (req.url) {
    case '/api/pong':
      console.log('pong!', INSTANCE_ID, PRIVATE_IP)
      try {
        throw new Error('')
      } catch (e) {
        console.log(e)
      }
      break
    case '/api/ping':
      console.log('ping!', INSTANCE_ID, PRIVATE_IP)
      const ips = await SomeThrow()
      someObjectLiteral.objectLiteralThrow()
      const others = ips.filter((ip) => ip !== PRIVATE_IP)

      others.forEach((ip) => {
        http.get(`http://[${ip}]:8080/api/pong`)
      })
      break
    case '/api/throw':
      someRandomThrow()
      break
  }

  res.end()
})

const wss = new WebSocketServer({ noServer: true })
