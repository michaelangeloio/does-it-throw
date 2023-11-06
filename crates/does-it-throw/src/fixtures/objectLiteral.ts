// @ts-nocheck
export const someObjectLiteral = {
  objectLiteralThrow({ someArg}: { someArg: string}) {
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
  someObjectLiteral.objectLiteralThrow({ someArg: 'hi'})
}

export const callToLiteral2 = () => {
	someObjectLiteral.objectLiteralThrow({ someArg: 'hi'})
}

export const callToLiteral3 = () => {
	someObjectLiteral.nestedObjectLiteral.nestedObjectLiteralThrow()
	SomeObject.someExampleThrow()
}
