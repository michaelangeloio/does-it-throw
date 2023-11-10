// @ts-nocheck

export const someConstThatThrows = () => {
	throw new Error('hi khue')
}

function callToConstThatThrows4() {
	someConstThatThrows()
}
