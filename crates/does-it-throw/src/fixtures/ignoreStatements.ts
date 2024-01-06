// @ts-nocheck
const someCondition = true
export class Something {
  constructor() {
    // @it-throws
    throw new Error('hi khue')
  }

  someMethodThatThrows() {
    // @it-throws
    throw new Error('hi khue')
  }

  someMethodThatDoesNotThrow() {
    console.log('hi khue')
  }

  someMethodThatThrows2() {
    if (someCondition) {
      // @some-random-ignore
      throw new Error('hi khue')
    }
  }

  nestedThrow() {
    if (someCondition) {
      return true
    }
    // @it-throws-ignore
    throw new Error('hi khue')
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


const someRandomThrow = () => {
  //@it-throws
  throw new Error('some random throw')
}

const server = http.createServer(async (req, res) => {
  switch (req.url) {
    case '/api/pong':
      console.log('pong!', INSTANCE_ID, PRIVATE_IP)
      //@it-throws
      throw new Error('')
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


function _somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}

export function somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}


// @ts-nocheck
const connection = {}

const SomeThrow = () => {
  //@it-throws
  throw new Error('hi khue')
}

function SomeThrow2() {
  //@it-throws
  throw new Error('hi khue')
}

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(DidChangeConfigurationNotification.type, undefined)
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log(`Workspace folder change event received. ${JSON.stringify(_event)}`)
    })
  }
  SomeThrow()
  SomeThrow2()
})

connection.onInitialized2(() => {
  //@it-throws
  throw new Error('hi khue')
})

SomeRandomCall(() => {
  //@it-throws
  throw new Error('hi khue')
})

SomeRandomCall2(() => {
  SomeThrow()
  SomeThrow2()
})

connection.oneWithASecondArg({}, () => {
  //@it-throws
  throw new Error('hi khue')
})
