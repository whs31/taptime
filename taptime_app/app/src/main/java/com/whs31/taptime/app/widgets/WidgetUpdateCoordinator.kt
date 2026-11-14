package com.whs31.taptime.app.widgets

import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.domain.TimelineMode
import com.whs31.taptime.app.rust.*
import java.time.Instant

class WidgetUpdateCoordinator(private val context: Context, private val app: AppContainer) {
    private val manager = AppWidgetManager.getInstance(context)
    private val prefs = WidgetPreferences(context)
    private val kinds: Map<WidgetKind, Class<out android.appwidget.AppWidgetProvider>> = mapOf(
        WidgetKind.Dashboard to DashboardWidgetProvider::class.java,
        WidgetKind.DayTimeline to DayTimelineWidgetProvider::class.java,
        WidgetKind.MonthBalance to MonthBalanceWidgetProvider::class.java,
        WidgetKind.MonthRing to MonthRingWidgetProvider::class.java,
        WidgetKind.Compact to CompactWidgetProvider::class.java,
        WidgetKind.Small to SmallWidgetProvider::class.java,
        WidgetKind.Tiny to TinyWidgetProvider::class.java,
    )
    fun hasMonthWidgets(): Boolean = listOf(WidgetKind.MonthBalance, WidgetKind.MonthRing).any {
        ids(it).isNotEmpty()
    }
    private fun ids(kind: WidgetKind) = manager.getAppWidgetIds(ComponentName(context, kinds.getValue(kind)))
    suspend fun updateAll() {
        for (kind in kinds.keys) updateKind(kind)
    }
    suspend fun updateKind(kind: WidgetKind) {
        val renderer = WidgetRenderer(context)
        val current = app.work.state.value
        val loggedIn = app.session.authenticated.value
        val day = current.day
        val monthReport = if (loggedIn && day != null && current.dashboard != null) {
            val start = java.time.LocalDate.parse(day.date).withDayOfMonth(1).toString()
            runCatching { app.gateway.buildReport(ReportInput(start, day.date, current.dashboard.days)) }.getOrNull()
        } else null
        val snapshot = if (loggedIn) current.metrics?.let { m ->
            com.whs31.taptime.app.data.DaySnapshotEntity(m.date, m.workedSeconds, m.balanceSeconds,
                current.monthlyBalanceSeconds ?: monthReport?.balanceSeconds ?: 0,
                m.firstCheckInSeconds, m.lastCheckOutSeconds, m.isWorking,
                m.requiredWorkSeconds, current.sampleEpoch)
        } ?: app.work.cached() else null
        for (id in ids(kind)) {
            val mode = prefs.mode(id, app.preferences.timelineMode)
            val timeline = if (kind == WidgetKind.DayTimeline && current.day != null && app.account.user.value != null) {
                runCatching { app.gateway.buildTimeline(TimelineInput(
                    DayCalculationInput(current.day, app.account.timeZone, Instant.now().epochSecond,
                        current.summary?.workTargetSeconds ?: current.day.requiredWorkSeconds,
                        current.summary?.beforeStartDate ?: false),
                    if (mode == TimelineMode.WORK_DAY) com.whs31.taptime.app.rust.TimelineMode.WORK_DAY
                    else com.whs31.taptime.app.rust.TimelineMode.CALENDAR_DAY)) }.getOrNull()
            } else null
            val state = WidgetState(loggedIn, snapshot, timeline, monthReport, current.stale, current.metrics?.remainingSeconds)
            manager.updateAppWidget(id, renderer.render(id, kind, state))
        }
    }
    fun forget(ids: IntArray) { ids.forEach { prefs.delete(it) } }
}
