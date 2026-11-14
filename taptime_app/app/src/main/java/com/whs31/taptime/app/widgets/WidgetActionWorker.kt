package com.whs31.taptime.app.widgets

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.whs31.taptime.app.TapTimeApplication

class WidgetActionWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val issuedAt = inputData.getLong("issued_at", -1)
        if (issuedAt <= 0 || System.currentTimeMillis() - issuedAt !in 0..10_000) return Result.failure()
        val app = (applicationContext as TapTimeApplication).container
        if (!app.session.authenticated.value && !app.session.restore()) return Result.failure()
        return try {
            if (app.account.user.value == null) app.account.refresh()
            if (System.currentTimeMillis() - issuedAt !in 0..10_000) return Result.failure()
            val connectivity = applicationContext.getSystemService(android.net.ConnectivityManager::class.java)
            val network = connectivity?.activeNetwork ?: return Result.failure()
            if (connectivity.getNetworkCapabilities(network)?.hasCapability(android.net.NetworkCapabilities.NET_CAPABILITY_VALIDATED) != true) return Result.failure()
            app.work.perform(inputData.getBoolean(WidgetActionReceiver.CHECK_IN, true))
            app.widgets.updateAll()
            Result.success()
        } catch (_: Exception) {
            app.widgets.updateAll()
            Result.failure()
        }
    }
}
