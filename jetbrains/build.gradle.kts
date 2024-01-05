import org.jetbrains.changelog.Changelog
import org.jetbrains.changelog.markdownToHTML

fun properties(key: String) = providers.gradleProperty(key)
fun environment(key: String) = providers.environmentVariable(key)

plugins {
    id("java") // Java support
    alias(libs.plugins.kotlin) // Kotlin support
    alias(libs.plugins.gradleIntelliJPlugin) // Gradle IntelliJ Plugin
    alias(libs.plugins.changelog) // Gradle Changelog Plugin
    alias(libs.plugins.qodana) // Gradle Qodana Plugin
    alias(libs.plugins.kover) // Gradle Kover Plugin
}

group = properties("pluginGroup").get()
version = properties("pluginVersion").get()

// Configure project's dependencies
repositories {
    mavenCentral()
}

tasks.test {
    useJUnitPlatform()
}

// Dependencies are managed with Gradle version catalog - read more: https://docs.gradle.org/current/userguide/platforms.html#sub:version-catalog
dependencies {
//    implementation(libs.annotations)
    testImplementation("org.junit.jupiter:junit-jupiter:5.8.1")

    // Mockito for mocking in tests
    testImplementation("org.mockito:mockito-core:4.5.1") // adjust the version as needed
    testImplementation("org.mockito.kotlin:mockito-kotlin:5.2.1")

}
// Set the JVM language level used to build the project. Use Java 11 for 2020.3+, and Java 17 for 2022.2+.
kotlin {
    @Suppress("UnstableApiUsage")
    jvmToolchain {
        languageVersion = JavaLanguageVersion.of(17)
        vendor = JvmVendorSpec.JETBRAINS
    }
}

intellij {
  pluginName = properties("pluginName")
  version = properties("platformVersion")
  type = properties("platformType")

  // Plugin Dependencies. Uses `platformPlugins` property from the gradle.properties file.
  plugins = properties("platformPlugins").map { it.split(',').map(String::trim).filter(String::isNotEmpty) }
}

// Configure Gradle Changelog Plugin - read more: https://github.com/JetBrains/gradle-changelog-plugin
changelog {
    groups.empty()
    repositoryUrl = properties("pluginRepositoryUrl")
}


// Configure Gradle Kover Plugin - read more: https://github.com/Kotlin/kotlinx-kover#configuration
koverReport {
    defaults {
        xml {
            onCheck = true
        }
    }
}

tasks {

  prepareSandbox {
      doLast {
          val pluginName = project.ext.get("pluginName") ?: throw GradleException("Plugin name not set.")

          val sourceDir = file("${project.projectDir}/../server/out")
          println("language server sourceDir: $sourceDir")
          val destDir = file("${destinationDir.path}/${pluginName}/language-server")
          println("language server destDir: $destDir")

          if (sourceDir.exists()) {
              copy {
                  from(sourceDir)
                  into(destDir)
              }
          } else {
              throw  GradleException("Source directory does not exist: $sourceDir")
          }
      }
  }

    wrapper {
        gradleVersion = properties("gradleVersion").get()
    }

    patchPluginXml {
        sinceBuild.set("233")
        untilBuild.set("240.*")

        // Extract the <!-- Plugin description --> section from README.md and provide for the plugin's manifest
        val readMe = layout.projectDirectory.file(file("${project.projectDir}/../README.md").path)
        println("readMe sourceDir: $readMe")
        pluginDescription = providers.fileContents(readMe).asText.map {
          val start1 = "<!-- JetBrains Plugin description 1 -->"
          val end1 = "<!-- JetBrains Plugin description end 1 -->"
          val start2 = "<!-- JetBrains Plugin description 2 -->"
          val end2 = "<!-- JetBrains Plugin description end 2 -->"
      
          val lines = it.lines()
      
          // Function to extract and convert content between start and end markers
          fun extractAndConvert(start: String, end: String): String {
              if (!lines.containsAll(listOf(start, end))) {
                  throw GradleException("Plugin description section not found in README.md:\n$start ... $end")
              }
              return lines.subList(lines.indexOf(start) + 1, lines.indexOf(end)).joinToString("\n").let(::markdownToHTML)
          }
      
          // Extract and convert both sections
          val content1 = extractAndConvert(start1, end1)
          val content2 = extractAndConvert(start2, end2)

          val readMeContent = "$content1\n$content2"
          println("readMe content: $readMeContent")
      
          // Combine both contents
          readMeContent
        }

        val changelog = project.changelog // local variable for configuration cache compatibility
        // Get the latest available change notes from the changelog file
        changeNotes = properties("pluginVersion").map { pluginVersion ->
            with(changelog) {
                renderItem(
                    (getOrNull(pluginVersion) ?: getUnreleased())
                        .withHeader(false)
                        .withEmptySections(true),
                    Changelog.OutputType.HTML,
                )
            }
        }
    }

    // Configure UI tests plugin
    // Read more: https://github.com/JetBrains/intellij-ui-test-robot
    runIdeForUiTests {
        systemProperty("robot-server.port", "8082")
        systemProperty("ide.mac.message.dialogs.as.sheets", "false")
        systemProperty("jb.privacy.policy.text", "<!--999.999-->")
        systemProperty("jb.consents.confirmation.enabled", "false")
    }

    signPlugin {
        certificateChain = environment("CERTIFICATE_CHAIN")
        privateKey = environment("PRIVATE_KEY")
        password = environment("PRIVATE_KEY_PASSWORD")
    }

    publishPlugin {
        dependsOn("patchChangelog")
        token = environment("JETBRAINS_PUBLISH_TOKEN")
        // The pluginVersion is based on the SemVer (https://semver.org) and supports pre-release labels, like 2.1.7-alpha.3
        // Specify pre-release label to publish the plugin in a custom Release Channel automatically. Read more:
        // https://plugins.jetbrains.com/docs/intellij/deployment.html#specifying-a-release-channel
    }

}
