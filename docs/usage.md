## Configuration Options

Here's a table of all the configuration options:

| Option | Description | Default |
| ------ | ----------- | ------- |
| `throwStatementSeverity` | The severity of the throw statement diagnostics. | `Hint` |
| `functionThrowSeverity` | The severity of the function throw diagnostics. | `Hint` |
| `callToThrowSeverity` | The severity of the call to throw diagnostics. | `Hint` |
| `callToImportedThrowSeverity` | The severity of the call to imported throw diagnostics. | `Hint` |
| `includeTryStatementThrows` | Whether to include throw statements inside try statements. | `false` |
| `maxNumberOfProblems` | The maximum number of problems to report. | `10000` |
| `ignoreStatements` | A list/array of statements to ignore. | `["@it-throws", "@does-it-throw-ignore"]` |

## Ignoring Throw Statement Warnings

You can ignore throw statement warnings by adding the following comment to the line above the throw statement:

```typescript
const someThrow = () => {
  // @does-it-throw-ignore
  throw new Error("This will not be reported");
};
```

Any calls to functions/methods that `throw` that are marked with the `@it-throws` or `@does-it-throw-ignore` comment will also be ignored as a result. For example:

```typescript
const someThrow = () => {
  // @does-it-throw-ignore
  throw new Error("This will not be reported");
};

const callToThrow = () => {
  someThrow(); // This will not be reported
};
```