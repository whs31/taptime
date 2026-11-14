import com.android.build.api.dsl.ApplicationExtension

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.kotlin.ksp) apply false
}

if (project == rootProject) {
val architectureCheck = tasks.register<Exec>("checkArchitecture") {
    commandLine("python3", "architecture/check_structure.py")
}

project(":app") {
    apply(plugin = "com.android.application")
    apply(plugin = "org.jetbrains.kotlin.android")
    apply(plugin = "org.jetbrains.kotlin.plugin.compose")
    apply(plugin = "com.google.devtools.ksp")

    extensions.configure<org.jetbrains.kotlin.gradle.dsl.KotlinAndroidExtension> {
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }

    extensions.configure<ApplicationExtension> {
        namespace = "com.whs31.taptime.app"
        compileSdk = 36
        defaultConfig {
            applicationId = "com.whs31.taptime.app"
            minSdk = 26
            targetSdk = 35
            versionCode = 1
            versionName = "0.1.0"
            ndk { abiFilters.addAll(listOf("arm64-v8a", "x86_64")) }
            testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        }
        buildFeatures { compose = true }
        testOptions { unitTests.isIncludeAndroidResources = true }
        compileOptions {
            sourceCompatibility = JavaVersion.VERSION_17
            targetCompatibility = JavaVersion.VERSION_17
        }
        sourceSets.getByName("main").java.srcDir(layout.buildDirectory.dir("generated/uniffi"))
        sourceSets.getByName("main").jniLibs.srcDir(layout.buildDirectory.dir("generated/rustJni"))
        packaging { jniLibs { useLegacyPackaging = true } }
    }

    dependencies {
        "implementation"(platform("androidx.compose:compose-bom:2025.10.01"))
        "implementation"("androidx.compose.ui:ui")
        "implementation"("androidx.compose.material3:material3")
        "implementation"("androidx.compose.material:material-icons-extended")
        "implementation"("androidx.compose.foundation:foundation")
        "implementation"("androidx.compose.ui:ui-tooling-preview")
        "debugImplementation"("androidx.compose.ui:ui-tooling")
        "debugImplementation"("androidx.compose.ui:ui-test-manifest")
        "implementation"("androidx.activity:activity-compose:1.11.0")
        "implementation"("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
        "implementation"("androidx.navigation:navigation-compose:2.9.5")
        "implementation"("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
        "implementation"("androidx.room:room-runtime:2.8.4")
        "implementation"("androidx.room:room-ktx:2.8.4")
        "ksp"("androidx.room:room-compiler:2.8.4")
        "implementation"("androidx.work:work-runtime-ktx:2.11.0")
        "implementation"("net.java.dev.jna:jna:5.17.0@aar")
        "testImplementation"("junit:junit:4.13.2")
        "testImplementation"("org.robolectric:robolectric:4.16")
        "testImplementation"("androidx.test:core:1.7.0")
        "testImplementation"("io.mockk:mockk:1.14.5")
        "testImplementation"("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.10.2")
        "androidTestImplementation"("androidx.test.ext:junit:1.3.0")
        "androidTestImplementation"("androidx.test.espresso:espresso-core:3.7.0")
        "androidTestImplementation"(platform("androidx.compose:compose-bom:2025.10.01"))
        "androidTestImplementation"("androidx.compose.ui:ui-test-junit4")
    }

    val rustManifest = rootProject.file("rust/Cargo.toml")
    val rustDir = rootProject.file("rust")
    val uniffiOut = layout.buildDirectory.dir("generated/uniffi")
    val rustHost = tasks.register<Exec>("buildRustHost") {
        workingDir = rustDir
        commandLine("cargo", "build", "--manifest-path", rustManifest.absolutePath)
    }
    val rustBindings = tasks.register<Exec>("generateRustBindings") {
        dependsOn(rustHost)
        workingDir = rustDir
        doFirst { uniffiOut.get().asFile.mkdirs() }
        commandLine(
            "uniffi-bindgen", "generate", "--library",
            rootProject.file("rust/target/debug/libtaptime_android_sdk.so").absolutePath,
            "--language", "kotlin", "--out-dir", uniffiOut.get().asFile.absolutePath,
        )
    }
    rustBindings.configure {
        doLast {
            val binding = uniffiOut.get().file("com/whs31/taptime/app/rust/taptime_android_sdk.kt").asFile
            val source = binding.readText()
            val corrected = source.replace("val `message`: kotlin.String,", "override val `message`: kotlin.String,")
                .replace(Regex("        override val message\\s+get\\(\\) = \"code=.*?\\n"), "")
            if (source == corrected || corrected.contains("        override val message\n")) error("UniFFI SdkError message adapter did not match generated Kotlin")
            binding.writeText(corrected)
        }
    }
    val rustAndroid = tasks.register<Exec>("buildRustAndroid") {
        workingDir = rustDir
        commandLine("cargo", "ndk", "-t", "arm64-v8a", "-t", "x86_64", "-o",
            layout.buildDirectory.dir("generated/rustJni").get().asFile.absolutePath,
            "build", "--release", "--manifest-path", rustManifest.absolutePath)
    }
    tasks.matching { it.name == "preBuild" }.configureEach { dependsOn(architectureCheck) }
    tasks.matching { it.name.startsWith("compile") && it.name.endsWith("Kotlin") }.configureEach {
        dependsOn(rustBindings)
    }
    tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach {
        dependsOn(rustAndroid)
    }
}

}
