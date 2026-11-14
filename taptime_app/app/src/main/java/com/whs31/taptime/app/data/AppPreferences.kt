package com.whs31.taptime.app.data

import android.content.Context
import com.whs31.taptime.app.domain.TimelineMode

class AppPreferences(context: Context) {
    private val store = context.getSharedPreferences("taptime-preferences", Context.MODE_PRIVATE)
    var timelineMode: TimelineMode
        get() = if (store.getString("timeline", "work") == "calendar") TimelineMode.CALENDAR_DAY else TimelineMode.WORK_DAY
        set(value) { store.edit().putString("timeline", if (value == TimelineMode.CALENDAR_DAY) "calendar" else "work").apply() }
    var dynamicColor: Boolean
        get() = store.getBoolean("dynamic_color", true)
        set(value) { store.edit().putBoolean("dynamic_color", value).apply() }
}
