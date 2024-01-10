package org.michaelangeloio.plugins.dit.settings

import com.intellij.openapi.components.service
import com.intellij.openapi.options.BoundSearchableConfigurable
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.DialogPanel
import com.intellij.ui.dsl.builder.*
import org.michaelangeloio.plugins.dit.DoesItThrowBundle
import org.michaelangeloio.plugins.dit.services.DoesItThrowServerService

class DoesItThrowSettingsConfigurable(private val project: Project) :
        BoundSearchableConfigurable(
                DoesItThrowBundle.message("does-it-throw.settings.name"),
                DoesItThrowBundle.message("does-it-throw.settings.name")
        ) {
    private val settings: DoesItThrowSettings = DoesItThrowSettings.getInstance(project)
    private val doesItThrowServerService = project.service<DoesItThrowServerService>()

    override fun createPanel(): DialogPanel = panel {


        row(DoesItThrowBundle.message("does-it-throw.settings.includeTryStatementThrows.label")) {
            checkBox("").bindSelected(settings::includeTryStatementThrows)
        }

        row(DoesItThrowBundle.message("does-it-throw.settings.maxNumberOfProblems.label")) {
            textField().bindIntText(settings::maxNumberOfProblems)
        }

        row("Ignore Statements:") {
            val textArea = textArea().bindText(
                    getter = { settings.ignoreStatements.joinToString("\n") },
                    setter = { text -> settings.ignoreStatements = text.split("\n").filter { it.isNotBlank() } }
            )
            textArea.rows(10) // Sets the number of visible rows in the text area.
        }

        group("Severity Configuration") {
            row{
                label("Allowed Values: Information, Hint, Warning, Error")
            }
            row(DoesItThrowBundle.message("does-it-throw.settings.throwStatementSeverity.label")) {
                textField().bindText(settings::throwStatementSeverity)
            }
            row (DoesItThrowBundle.message("does-it-throw.settings.functionThrowSeverity.label")) {
                textField().bindText(settings::functionThrowSeverity)
            }
            row (DoesItThrowBundle.message("does-it-throw.settings.callToThrowSeverity.label")) {
                textField().bindText(settings::callToThrowSeverity)
            }
            row (DoesItThrowBundle.message("does-it-throw.settings.callToImportedThrowSeverity.label")) {
                textField().bindText(settings::callToImportedThrowSeverity)
            }
        }

        onApply {
            doesItThrowServerService.restartDoesItThrowServer()
            doesItThrowServerService.notifyRestart()
        }
    }
}
