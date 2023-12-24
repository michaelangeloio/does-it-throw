//@ts-nocheck
async function throwInsideCatch() {
  try {
    throw new Error('hi')
  } catch (e) {
    throw e
  }
}

async function parentCatchThatisNotCaught() {
  try {
    try {
      something()
    }
    catch (e) {
      throw new Error()
    }
  } catch (e)  {
    throw new Error()
  }
}

async function noThrowInsideCatch() {
  try {
    throw new Error('hi')
  } catch (e) {
    console.log(e)
  }
}

async function parentCatchWithoutThrow() {
  try {
    throw new Error('hi')
  } catch (e) {
    try {
      throw new Error('hi')
    } catch (e) {
      console.log(e)
    }
  }
}