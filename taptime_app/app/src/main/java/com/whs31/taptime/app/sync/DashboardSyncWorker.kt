package com.whs31.taptime.app.sync

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import com.whs31.taptime.app.TapTimeApplication
import android.util.Log

class DashboardSyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val app = applicationContext as TapTimeApplication
        val container = app.container
        if (!container.session.authenticated.value && !container.session.restore()) return Result.success()
        return try {
            val state = container.sync.refresh()
            if (state.stale) Result.retry() else Result.success()
        } catch (error: Exception) {
            Log.w(TAG, "refresh_failed type=${error.javaClass.simpleName}")
            Result.retry()
        }
    }

    private companion object { const val TAG = "TapTimeSyncWorker" }
}
