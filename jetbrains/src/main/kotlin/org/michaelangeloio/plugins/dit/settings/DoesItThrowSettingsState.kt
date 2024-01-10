package org.michaelangeloio.plugins.dit.settings

import com.intellij.openapi.components.BaseState
import com.intellij.openapi.components.Service
import org.jetbrains.annotations.ApiStatus

@Service
@ApiStatus.Internal
class DoesItThrowSettingsState : BaseState() {
    var throwStatementSeverity by string("Hint")
    var functionThrowSeverity by string("Hint")
    var callToThrowSeverity by string("Hint")
    var callToImportedThrowSeverity by string("Hint")
    var includeTryStatementThrows by property(false)
    var maxNumberOfProblems by property(1000)
    var ignoreStatements = listOf(
            "@it-throws",
            "@does-it-throw-ignore"
    )
}
