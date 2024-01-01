package org.michaelangeloio.plugins.dit

import com.intellij.openapi.vfs.VirtualFile

object DoesItThrowUtils {
    fun isSupportedFileType(file: VirtualFile): Boolean = when (file.extension) {
        "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "d.ts", "json", "jsonc" -> true
        else -> false
    }
}
