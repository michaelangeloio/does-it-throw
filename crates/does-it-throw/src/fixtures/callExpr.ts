// @ts-nocheck
const connection = {}


const SomeThrow = () => {
	throw new Error('hi khue')
}

function SomeThrow2() {
	throw new Error('hi khue')
}

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(DidChangeConfigurationNotification.type, undefined)
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log(`Workspace folder change event received. ${JSON.stringify(_event)}`)
    })
  }
  SomeThrow()
	SomeThrow2()
})


connection.onInitialized2(() => {
	throw new Error('hi khue')
})


SomeRandomCall(() => {
	throw new Error('hi khue')
})

SomeRandomCall2(() => {
	SomeThrow()
	SomeThrow2()
})

connection.oneWithASecondArg({}, () => {
	throw new Error('hi khue')
})
