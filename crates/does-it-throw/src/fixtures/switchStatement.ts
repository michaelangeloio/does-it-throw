//@ts-nocheck

import { SomeThrow, someObjectLiteral } from "./something"

const someRandomThrow = () => {
  throw new Error('some random throw')
}

export const server = http.createServer(async (req, res) => {

  switch (req.url) {
    case '/api/pong':
      console.log('pong!', INSTANCE_ID, PRIVATE_IP)
      throw new Error('')
      break
    case '/api/ping':
      console.log('ping!', INSTANCE_ID, PRIVATE_IP)
      const ips = await SomeThrow()
      someObjectLiteral.objectLiteralThrow()
      const others = ips.filter(ip => ip !== PRIVATE_IP)

      others.forEach(ip => {
        http.get(`http://[${ip}]:8080/api/pong`)
      })
      break
    case '/api/throw':
      someRandomThrow()
      break
  }

  res.end()
})

const wss = new WebSocketServer({ noServer: true })
