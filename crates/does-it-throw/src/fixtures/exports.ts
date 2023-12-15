// @ts-nocheck

type SomeRandomType = {
  hiKhue: string
}

export function hiKhue({ hiKhue }: { hiKhue: string }) {
  throw new Error('hi khue')
}

export const someConstThatThrows = () => {
  throw new Error('hi khue')
}

const _ConstThatDoesNotThrow = ({
  someCondition
}: {
  someCondition: {
    hiKhue: string
  }
}) => {
  console.log('hi khue')
  someCondition.hiKhue
}

const _ConstThatThrows = () => {
  throw new Error('hi khue')
}

const callToConstThatThrows = () => {
  someConstThatThrows()
}

export const someConstThatThrows2 = () => {
  if (someCondition) {
    throw new Error('hi khue')
  }
}

export const callToConstThatThrows2 = () => {
  someConstThatThrows2()
}

export function callToConstThatThrows3() {
  const hello: SomeRandomType = {
    hiKhue: 'hi khue'
  }
  someConstThatThrows2()
}

function callToConstThatThrows4() {
  someConstThatThrows2()
}
