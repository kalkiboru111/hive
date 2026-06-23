// Minimal sbt project for the Hive rApp deploy tool.
// The Reality SDK is provided as an unmanaged jar: drop the reality-combined
// assembly jar at deploy/lib/reality-combined.jar (sbt auto-includes lib/*.jar).
// See README.md.
ThisBuild / scalaVersion := "3.7.4"

lazy val root = (project in file("."))
  .settings(
    name := "hive-deploy",
    run / fork := true,
    // lib/*.jar is included by default (unmanagedBase = baseDirectory/lib),
    // but be explicit so it's obvious where the SDK comes from:
    Compile / unmanagedJars ++= {
      val libDir = baseDirectory.value / "lib"
      (libDir ** "*.jar").classpath
    }
  )
