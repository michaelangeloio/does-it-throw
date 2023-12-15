//@ts-nocheck
import { resolve } from 'path'
import { SomeThrow, SomeThrow as SomeThrow2, Something, testing as Test, testing as Test2 } from './something'
import Testing from './something3'
import { SomethingElse } from './somethingElse'
import { SomethingElse as SomethingElse2 } from './somethingElse2'

export function test() {
  try {
    SomethingElse()
  } catch (e) {
    console.log(e)
  }
  try {
    SomethingElse2()
  } catch (e) {
    console.log(e)
  }
  try {
    Testing()
  } catch (e) {
    console.log(e)
  }
  resolve()
  try {
    SomeThrow()
  } catch (e) {
    console.log(e)
  }
  try {
    Test()
  } catch (e) {
    console.log(e)
  }
  try {
    SomeThrow2()
  } catch (e) {
    console.log(e)
  }
  try {
    Test2()
  } catch (e) {
    console.log(e)
  }
  try {
    Something()
  } catch (e) {
    console.log(e)
  }
}
