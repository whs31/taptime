package com.whs31.taptime.app.widgets

import android.content.Context
import com.whs31.taptime.app.domain.TimelineMode

class WidgetPreferences(context: Context) {
    private val store = context.getSharedPreferences("taptime-widgets", Context.MODE_PRIVATE)
    fun mode(widgetId: Int, default: TimelineMode): TimelineMode =
        when (store.getString("mode-$widgetId", null)) {
            "calendar" -> TimelineMode.CALENDAR_DAY
            "work" -> TimelineMode.WORK_DAY
            else -> default
        }
    fun setMode(widgetId: Int, mode: TimelineMode) =
        store.edit().putString("mode-$widgetId", if (mode == TimelineMode.CALENDAR_DAY) "calendar" else "work").apply()
    fun delete(widgetId: Int) = store.edit().remove("mode-$widgetId").apply()
}
