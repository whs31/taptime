package com.whs31.taptime.app.widgets

import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Context
import android.util.Log
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.data.DaySnapshotEntity
import com.whs31.taptime.app.domain.TimelineMode
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class WidgetUpdateCoordinator(private val context: Context, private val app: AppContainer) {
    private val manager = AppWidgetManager.getInstance(context)
    private val prefs = WidgetPreferences(context)
    private val updateMutex = Mutex()
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

    suspend fun updateAll() = updateMutex.withLock {
        val snapshot = capture()
        Log.d(TAG, "widget_batch sample=${snapshot.state.snapshot?.sampledAtEpochSeconds ?: 0} stale=${snapshot.state.stale} active=${snapshot.state.snapshot?.isWorking == true}")
        kinds.keys.forEach { kind -> updateKind(kind, snapshot) }
    }

    suspend fun updateKind(kind: WidgetKind) = updateMutex.withLock {
        updateKind(kind, capture())
    }

    private suspend fun capture(): RenderSnapshot {
        val current = app.work.state.value
        val loggedIn = app.session.authenticated.value
        val report = if (loggedIn && current.dashboard != null) {
            runCatching {
                app.gateway.buildReport(ReportInput(
                    current.dashboard.days.firstOrNull()?.day?.date ?: return@runCatching null,
                    current.dashboard.days.lastOrNull()?.day?.date ?: return@runCatching null,
                    current.dashboard.days,
                ))
            }.getOrNull()
        } else {
            null
        }
        val snapshot = if (loggedIn) {
            current.metrics?.let { metrics ->
                DaySnapshotEntity(
                    metrics.date,
                    metrics.workedSeconds,
                    metrics.balanceSeconds,
                    current.monthlyBalanceSeconds ?: report?.balanceSeconds ?: 0,
                    metrics.firstCheckInSeconds,
                    metrics.lastCheckOutSeconds,
                    metrics.isWorking,
                    metrics.requiredWorkSeconds,
                    current.sampleEpoch,
                )
            } ?: app.work.cached()
        } else {
            null
        }
        return RenderSnapshot(
            WidgetState(loggedIn, snapshot, null, report, current.stale, current.metrics?.remainingSeconds),
            current.day,
            current.summary,
            app.account.user.value?.timeZone,
            current.sampleEpoch,
        )
    }

    private fun updateKind(kind: WidgetKind, snapshot: RenderSnapshot) {
        val renderer = WidgetRenderer(context)
        for (id in ids(kind)) {
            val mode = prefs.mode(id, app.preferences.timelineMode)
            val timeline = if (kind == WidgetKind.DayTimeline && snapshot.day != null && snapshot.timeZone != null && snapshot.sampleEpoch > 0) {
                runCatching {
                    app.gateway.buildTimeline(TimelineInput(
                        DayCalculationInput(
                            snapshot.day,
                            snapshot.timeZone,
                            snapshot.sampleEpoch,
                            snapshot.summary?.workTargetSeconds ?: snapshot.day.requiredWorkSeconds,
                            snapshot.summary?.beforeStartDate ?: false,
                        ),
                        if (mode == TimelineMode.WORK_DAY) com.whs31.taptime.app.rust.TimelineMode.WORK_DAY
                        else com.whs31.taptime.app.rust.TimelineMode.CALENDAR_DAY,
                    ))
                }.getOrNull()
            } else {
                null
            }
            manager.updateAppWidget(id, renderer.render(id, kind, WidgetState(
                snapshot.state.authenticated,
                snapshot.state.snapshot,
                timeline,
                snapshot.state.report,
                snapshot.state.stale,
                snapshot.state.remainingSeconds,
            )))
        }
    }

    fun forget(ids: IntArray) {
        ids.forEach { prefs.delete(it) }
    }

    private data class RenderSnapshot(
        val state: WidgetState,
        val day: DayDto?,
        val summary: DaySummaryDto?,
        val timeZone: String?,
        val sampleEpoch: Long,
    )

    private companion object {
        const val TAG = "TapTimeWidgetUpdate"
    }
}
