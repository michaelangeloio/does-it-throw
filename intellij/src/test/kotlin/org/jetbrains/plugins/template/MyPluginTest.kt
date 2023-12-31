package org.intellij.prisma.lsp

import com.intellij.lang.javascript.modules.JSTempDirWithNodeInterpreterTest
import com.intellij.platform.lsp.tests.checkLspHighlighting

class DoesItThrowLspHighlightingTest : JSTempDirWithNodeInterpreterTest() {
  fun testCreateEnumQuickFix() {
    myFixture.configureByText("foo.prisma", """
      model User {
        name <error descr="Type \"Foo\" is neither a built-in type, nor refers to another model, custom type, or enum.">Foo</error><caret>
      }

    """.trimIndent())
    myFixture.checkLspHighlighting()
    myFixture.launchAction(myFixture.findSingleIntention("Create new enum 'Foo'"))
    myFixture.checkResult("""
      model User {
        name Foo<caret>
      }

      enum Foo {

      }

    """.trimIndent())
  }
}