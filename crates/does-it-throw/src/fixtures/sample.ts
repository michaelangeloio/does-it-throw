// @ts-nocheck

export const someConstThatThrows = () => {
  // @it-throws
  throw new Error('hi khue')
}

function callToConstThatThrows4() {
  someConstThatThrows()
}
