package org.michaelangeloio.plugins.dit.lsp

import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.javascript.nodejs.interpreter.NodeCommandLineConfigurator
import com.intellij.javascript.nodejs.interpreter.NodeJsInterpreterManager
import com.intellij.javascript.nodejs.interpreter.local.NodeJsLocalInterpreter
import com.intellij.javascript.nodejs.interpreter.wsl.WslNodeInterpreter
import com.intellij.lang.javascript.service.JSLanguageServiceUtil
import com.intellij.openapi.diagnostic.thisLogger
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import org.michaelangeloio.plugins.dit.DoesItThrowUtils

class DoesItThrowLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "Does it Throw?") {

    override fun isSupportedFile(file: VirtualFile) = DoesItThrowUtils.isSupportedFileType(file)

    override fun createCommandLine(): GeneralCommandLine {
        val interpreter = NodeJsInterpreterManager.getInstance(project).interpreter
        if (interpreter !is NodeJsLocalInterpreter && interpreter !is WslNodeInterpreter) {
            // shouldn't happen
            throw ExecutionException("no local node interpreter ")
        }

        val lsp = JSLanguageServiceUtil.getPluginDirectory(javaClass, "language-server/server.js")
        thisLogger().info("language server loaded")
        thisLogger().info(lsp.path)
        if (lsp == null || !lsp.exists()) {
            // broken plugin installation?
            throw ExecutionException("could not find language server")
        }

        return GeneralCommandLine().apply {
            withParentEnvironmentType(GeneralCommandLine.ParentEnvironmentType.CONSOLE)
            withCharset(Charsets.UTF_8)
            addParameter(lsp.path)
            addParameter("--stdio")

            NodeCommandLineConfigurator.find(interpreter)
                    .configure(this, NodeCommandLineConfigurator.defaultOptions(project))
        }
    }

  // references resolution is implemented without using the LSP server
  override val lspGoToDefinitionSupport = false

  // code completion is implemented without using the LSP server
  override val lspCompletionSupport = null

  // code formatting is implemented without using the LSP server
  override val lspFormattingSupport = null
}
