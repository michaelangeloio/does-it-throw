//@ts-nocheck
import { testing as Test } from './something'

import { SomeObject } from './something123'

console.log('\x1b[36m%s\x1b[0m', Test)

export function hiKhue() {
  throw new Error('hi khue')
}

export class Something {
  constructor() {}

  someMethodThatThrows() {
    throw new Error('hi khue')
  }

  someMethodThatDoesNotThrow() {
    console.log('hi khue')
  }

  someMethodThatThrows2() {
    if (something) {
      throw new Error('hi khue')
    }
    if (something) {
      throw new Error('hi khue')
    }
  }

  nestedThrow() {
    if (somethingRandom) {
      return true
    }
    throw new Error('hi khue')
  }

  callNestedThrow() {
    if (somethingRandom) {
      return true
    }
    if (somethingRandom2) {
      return true
    }
    this.nestedThrow()
  }

  callImportedThrow() {
    SomeObject.someImportedThrow()
  }
}

export function somethingElse() {
  const something = new Something()
  something.someMethodThatThrows()
}

export function callHiKhue() {
  hiKhue()
}

export const someObjectLiteral = {
  objectLiteralThrow() {
    throw new Error('hi khue')
  },
  nestedObjectLiteral: {
    nestedObjectLiteralThrow: () => {
      throw new Error('hi khue')
    },
  },
}

export const SomeObject = {
  someExampleThrow: () => {
    throw new Error('hi khue')
  },
}

export const someConstThatThrows = () => {
  throw new Error('hi khue')
}

export function callToLiteral() {
  someObjectLiteral.objectLiteralThrow()
}

const connection = {}

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
  throw new Error('')
})
