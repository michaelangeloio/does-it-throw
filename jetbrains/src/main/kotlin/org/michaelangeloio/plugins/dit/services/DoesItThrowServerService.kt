package org.michaelangeloio.plugins.dit.services

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.components.Service
import com.intellij.openapi.project.Project
import com.intellij.platform.lsp.api.LspServerManager
import org.michaelangeloio.plugins.dit.DoesItThrowBundle
import org.michaelangeloio.plugins.dit.lsp.DoesItThrowLspServerSupportProvider

@Service(Service.Level.PROJECT)
class DoesItThrowServerService(private val project: Project) {

    fun restartDoesItThrowServer() {
        LspServerManager.getInstance(project).stopAndRestartIfNeeded(DoesItThrowLspServerSupportProvider::class.java)
    }

    fun notifyRestart() {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("DoesItThrow")
            .createNotification(
                DoesItThrowBundle.message("does-it-throw.language.server.restarted"),
                "",
                NotificationType.INFORMATION
            )
            .notify(project)
    }
}
