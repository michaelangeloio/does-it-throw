export const someThrow = () => {
  throw new Error('some error')
}
export function someThrow2() {
	throw new Error('some error')
}

export const someTsx = () => {
  if (something) {
    throw new Error()
  }
  return <div>some tsx</div>
}

export async function someAsyncTsx() {
  if (something) {
    throw new Error()
  }
  return <div>some tsx</div>
}

export async function callToThrow() {
  someThrow()
	someThrow2()
  return <div>some tsx</div>
}

export const someTsxWithJsx = async () => {
	someThrow()
	someThrow2()
	return <div>some tsx</div>
}
