// @ts-nocheck
const someThrow = () => {
  if (something) {
    while (true) {
      throw new Error("oh no");
    }
  } else {
    for (let i = 0; i < 10; i++) {
      throw new Error("oh no");
    }
  }
}
class Test {
  badMethod() {
    throw new Error("oh no");
  }
}

const callToSomeThrow = () => {
  const testMethod = new Test();
  return {
    test: someThrow(),
    testing: () => someThrow(),
    array: [someThrow(), someThrow()],
    object: { test: someThrow() },
    class: testMethod.badMethod(),
  }
}

function test() {
  return someThrow();
}