package com.whs31.taptime.app.widgets

import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import com.whs31.taptime.app.TapTimeApplication
import kotlinx.coroutines.launch

class MonthRingWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, manager: AppWidgetManager, ids: IntArray) {
        val app = (context.applicationContext as TapTimeApplication).container
        val pending = goAsync()
        app.scope.launch {
            try {
            androidx.work.WorkManager.getInstance(context).enqueueUniqueWork(
                "taptime-widget-refresh", androidx.work.ExistingWorkPolicy.KEEP,
                androidx.work.OneTimeWorkRequestBuilder<com.whs31.taptime.app.sync.DashboardSyncWorker>()
                    .setConstraints(androidx.work.Constraints.Builder().setRequiredNetworkType(androidx.work.NetworkType.CONNECTED).build())
                    .build())
            app.widgets.updateKind(WidgetKind.MonthRing)
            } finally { pending?.finish() }
        }
    }
    override fun onAppWidgetOptionsChanged(context: Context, manager: AppWidgetManager, id: Int, options: android.os.Bundle) {
        onUpdate(context, manager, intArrayOf(id))
    }
    override fun onDeleted(context: Context, ids: IntArray) {
        (context.applicationContext as TapTimeApplication).container.widgets.forget(ids)
    }
}
