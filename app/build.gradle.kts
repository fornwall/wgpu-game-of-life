plugins {
    id("com.android.application")
}

android {
    namespace = "net.fornwall.wgpugameoflife"
    ndkVersion = "26.2.11394342"
    compileSdk = 34

    defaultConfig {
        minSdk = 28
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
        debug {
            isMinifyEnabled = false
            //packagingOptions {
            //    doNotStrip "**/*.so"
            //}
            //debuggable true
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

fun runCargo(release: Boolean): Array<String> {
    var parameters = arrayOf("cargo", "ndk", "-t", "arm64-v8a", "-t", "x86_64", "-o", "app/src/main/jniLibs/", "build")
    if (release) parameters += "--release"
    return parameters
}

task<Exec>("buildRustDebug") {
    workingDir = file("..")
    commandLine(*runCargo(false))
}

task<Exec>("buildRustRelease") {
    workingDir = file("..")
    commandLine(*runCargo(true))
}

tasks.whenTaskAdded {
    if (name == "packageDebug") {
        dependsOn("buildRustDebug")
    } else if (name == "packageRelease") {
        dependsOn("buildRustRelease")
    }
}

