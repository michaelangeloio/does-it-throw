//@ts-nocheck
import { testing as Test } from './something'

import { SomeObject } from './something123'

console.log('\x1b[36m%s\x1b[0m', Test)

export function hiKhue() {
	throw new Error('hi khue')
}

export class Something {
	constructor() {
	}

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

	nestedThrow () {
		if (somethingRandom) {
			return true
		}
		throw new Error('hi khue')
	}

	callNestedThrow () {
		if (somethingRandom) {
			return true
		}
		if (somethingRandom2) {
			return true
		}
		this.nestedThrow()
	}

	callImportedThrow () {
		SomeObject.someImportedThrow()
	}

}


export function somethingElse () {
	const something = new Something()
	something.someMethodThatThrows()
}


export function callHiKhue () {
	hiKhue()
}
