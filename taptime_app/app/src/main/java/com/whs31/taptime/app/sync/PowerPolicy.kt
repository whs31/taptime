package com.whs31.taptime.app.sync

import android.content.Context
import android.os.PowerManager

class PowerPolicy(context: Context) {
    private val power = context.getSystemService(PowerManager::class.java)
    val isSaving: Boolean get() = power?.isPowerSaveMode == true || power?.isDeviceIdleMode == true
    fun backgroundMinutes(): Long = if (isSaving) 30 else 15
    fun foregroundMinutes(): Long = if (isSaving) 15 else 5
}
