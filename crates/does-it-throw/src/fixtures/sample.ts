// @ts-nocheck

export const someConstThatThrows = () => {
  throw new Error('hi khue')
}

function callToConstThatThrows4() {
  someConstThatThrows()
}


export const someConstThatDoesNotThrow = () => {
  console.log('hi khue')
}