// @ts-nocheck
const someCondition = true
export class Something {
  constructor() {
    throw new Error('hi khue')
  }

  someMethodThatThrows() {
    throw new Error('hi khue')
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
    if (someCondition) {
      return true
    }
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

function _somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}

export function somethingCall2() {
  const something = new Something()
  something.someMethodThatThrows()
}
