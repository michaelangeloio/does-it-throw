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
  }
}

export function callToLiteral() {
  someObjectLiteral.objectLiteralThrow()
}

export const callToLiteral2 = () => {
	someObjectLiteral.objectLiteralThrow()
}

export const callToLiteral3 = () => {
	someObjectLiteral.nestedObjectLiteral.nestedObjectLiteralThrow()
	SomeObject.someExampleThrow()
}
