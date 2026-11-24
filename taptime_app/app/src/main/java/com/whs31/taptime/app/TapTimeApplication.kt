package com.whs31.taptime.app

import android.app.Application

class TapTimeApplication : Application() {
    val container: AppContainer by lazy { AppContainer(this) }
    override fun onCreate() {
        super.onCreate()
        container.initialize()
    }
}
