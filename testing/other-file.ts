export class SomethingThatThrows2 {
    constructor() {
    }
    doSomething() {
        throw new Error('something');
    }
}
