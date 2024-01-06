import { ESLintUtils } from '@typescript-eslint/utils';

const createRule = ESLintUtils.RuleCreator(
  name => `https://example.com/rule/${name}`,

);

// Type: RuleModule<"uppercase", ...>
export const rule = createRule({
  create(context) {
    return {
      FunctionDeclaration(node) {
        if (node.id != null) {
          if (/^[a-z]/.test(node.id.name)) {
            context.report({
              messageId: 'uppercase',
              node: node.id,
            });
            context.report({
              messageId
              loc: {

              }
            })
          }
        }
      },
    };
  },
  name: 'uppercase-first-declarations',
  meta: {
    docs: {
      description:
        'Function declaration names should start with an upper-case letter.',
    },
    messages: {
      uppercase: 'Start this name with an upper-case letter.',
    },b
    type: 'suggestion',
    schema: [],
  },
  defaultOptions: [],
});