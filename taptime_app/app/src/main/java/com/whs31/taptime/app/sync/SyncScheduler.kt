package com.whs31.taptime.app.sync

import android.content.Context
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import java.util.concurrent.TimeUnit

class SyncScheduler(private val context: Context, private val policy: PowerPolicy) {
    fun start() {
        val request = PeriodicWorkRequestBuilder<DashboardSyncWorker>(policy.backgroundMinutes(), TimeUnit.MINUTES)
            .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
            .build()
        WorkManager.getInstance(context).enqueueUniquePeriodicWork("taptime-sync", ExistingPeriodicWorkPolicy.UPDATE, request)
    }
    fun stop() = WorkManager.getInstance(context).cancelUniqueWork("taptime-sync")
}
