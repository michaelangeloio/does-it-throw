import com.intellij.openapi.vfs.VirtualFile
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.michaelangeloio.plugins.dit.DoesItThrowUtils
import org.mockito.kotlin.mock
import org.mockito.kotlin.whenever

class DoesItThrowUtilsTest {

    private val mockFile: VirtualFile = mock()

    @Test
    fun `should return true for supported file extensions`() {
        val supportedExtensions = listOf("js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "d.ts", "json", "jsonc")
        supportedExtensions.forEach { extension ->
            whenever(mockFile.extension).thenReturn(extension)
            assertTrue(DoesItThrowUtils.isSupportedFileType(mockFile))
        }
    }

    @Test
    fun `should return false for unsupported file extensions`() {
        whenever(mockFile.extension).thenReturn("unsupported")
        assertFalse(DoesItThrowUtils.isSupportedFileType(mockFile))
    }

    @Test
    fun `should return false for null or empty extensions`() {
        whenever(mockFile.extension).thenReturn(null)
        assertFalse(DoesItThrowUtils.isSupportedFileType(mockFile))

        whenever(mockFile.extension).thenReturn("")
        assertFalse(DoesItThrowUtils.isSupportedFileType(mockFile))
    }

    @Test
    fun `should correctly handle case sensitivity`() {
        // Assuming the function is case-insensitive
        whenever(mockFile.extension).thenReturn("JS")
        assertFalse(DoesItThrowUtils.isSupportedFileType(mockFile))
    }
}
