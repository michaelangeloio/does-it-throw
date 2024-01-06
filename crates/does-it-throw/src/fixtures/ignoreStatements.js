// @ts-nocheck
const someCondition = true
export class Something {
  constructor() {
    //@it-throws
    throw new Error('hi khue')
  }

  someMethodThatThrows() {
    //@it-throws
    throw new Error('hi khue')
  }

  someMethodThatDoesNotThrow() {
    console.log('hi khue')
  }

  someMethodThatThrows2() {
    if (someCondition) {
      //@it-throws
      throw new Error('hi khue')
    }
  }

  nestedThrow() {
    if (someCondition) {
      return true
    }
    //@it-throws
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
