package com.whs31.taptime.app.widgets

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import androidx.work.*

class WidgetActionReceiver : BroadcastReceiver() {
    companion object {
        const val ACTION = "com.whs31.taptime.app.WIDGET_WORK_ACTION"
        const val WIDGET_ID = "widget_id"
        const val CHECK_IN = "check_in"
    }
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != ACTION) return
        val connectivity = context.getSystemService(ConnectivityManager::class.java)
        val network = connectivity?.activeNetwork ?: return
        val capabilities = connectivity.getNetworkCapabilities(network) ?: return
        if (!capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)) return
        val widgetId = intent.getIntExtra(WIDGET_ID, -1)
        if (widgetId < 0) return
        val input = workDataOf(CHECK_IN to intent.getBooleanExtra(CHECK_IN, true),
            "issued_at" to System.currentTimeMillis())
        val work = OneTimeWorkRequestBuilder<WidgetActionWorker>().setInputData(input)
            .setExpedited(OutOfQuotaPolicy.DROP_WORK_REQUEST).build()
        WorkManager.getInstance(context).enqueueUniqueWork("taptime-user-action", ExistingWorkPolicy.KEEP, work)
    }
}
