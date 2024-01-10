package org.michaelangeloio.plugins.dit.settings

import com.intellij.openapi.components.*
import com.intellij.openapi.project.Project

//class StringMutableProperty(private var value: String?) : MutableProperty<String?> {
//    override fun get(): String? = value
//
//    override fun set(value: String?) {
//        this.value = value
//    }
//}

@Service(Service.Level.PROJECT)
@State(name = "DoesItThrowSettings", storages = [(Storage("does-it-throw.xml"))])
class DoesItThrowSettings :
    SimplePersistentStateComponent<DoesItThrowSettingsState>(DoesItThrowSettingsState()) {
//    private val throwStatementSeverityProperty = StringMutableProperty(throwStatementSeverity)
//
var throwStatementSeverity: String
    get() = state.throwStatementSeverity ?: "Hint"
    set(value) {
        state.throwStatementSeverity = value
    }

    var functionThrowSeverity: String
        get() = state.functionThrowSeverity ?: "Hint"
        set(value) {
            state.functionThrowSeverity = value
        }
    var callToThrowSeverity: String
        get() = state.callToThrowSeverity ?: "Hint"
        set(value) {
            state.callToThrowSeverity = value
        }
    var callToImportedThrowSeverity: String
        get() = state.callToImportedThrowSeverity ?: "Hint"
        set(value) {
            state.callToImportedThrowSeverity = value
        }
    var includeTryStatementThrows: Boolean

        get() = state.includeTryStatementThrows ?: false
        set(value) {
            state.includeTryStatementThrows = value
        }
    var maxNumberOfProblems: Int
        get() = state.maxNumberOfProblems ?: 1000
        set(value) {
            state.maxNumberOfProblems = value
        }
    var ignoreStatements: List<String>
        get() = state.ignoreStatements ?: listOf(
            "@it-throws",
            "@does-it-throw-ignore"
        )
        set(value) {
            state.ignoreStatements = value
        }

    companion object {
        @JvmStatic
        fun getInstance(project: Project): DoesItThrowSettings = project.service()
    }
}
