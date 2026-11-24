package com.whs31.taptime.app.data

import android.os.SystemClock
import com.whs31.taptime.app.domain.WorkActionCoordinator
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.time.Instant
import java.time.LocalDate
import java.time.YearMonth
import java.time.ZoneId

class WorkRepository(private val gateway: TapTimeGateway, private val account: AccountRepository,
                     private val actions: WorkActionCoordinator, private val cache: SnapshotDao) {
    data class WorkState(val metrics: DayMetricsDto? = null, val dashboard: DashboardDto? = null,
        val summary: DaySummaryDto? = null, val day: DayDto? = null,
        val monthlyBalanceSeconds: Long? = null,
        val sampleEpoch: Long = 0, val loading: Boolean = false, val stale: Boolean = true,
        val error: String? = null)
    private val _state = MutableStateFlow(WorkState())
    val state = _state.asStateFlow()
    private val refreshMutex = Mutex()
    private val stateLock = Any()
    private var sampleElapsedRealtimeMillis: Long? = null

    suspend fun refreshToday(): WorkState = refreshMutex.withLock { refreshTodayLocked() }

    private suspend fun refreshTodayLocked(): WorkState {
        publish(current().copy(loading = true))
        try {
            val user = account.user.value ?: account.refresh()
            val today = LocalDate.now(ZoneId.of(user.timeZone))
            val start = YearMonth.from(today).atDay(1)
            val request = DashboardQuery(start.toString(), today.toString(), start.toString(), today.toString(), today.toString())
            val dashboard = gateway.fetchDashboard(request)
            val summary = dashboard.days.find { it.day.date == today.toString() }
            val day = summary?.day ?: gateway.fetchDay(today.toString())
            val now = Instant.now().epochSecond
            val metrics = gateway.calculateDay(DayCalculationInput(day, user.timeZone, now,
                summary?.workTargetSeconds ?: day.requiredWorkSeconds, summary?.beforeStartDate ?: false))
            val monthly = gateway.buildReport(ReportInput(start.toString(), today.toString(), dashboard.days))
            val refreshed = WorkState(metrics, dashboard, summary, day, monthly.balanceSeconds, now, loading = false, stale = false)
            synchronized(stateLock) {
                sampleElapsedRealtimeMillis = SystemClock.elapsedRealtime()
                _state.value = refreshed
            }
            cache.upsert(DaySnapshotEntity(today.toString(), metrics.workedSeconds, metrics.balanceSeconds,
                monthly.balanceSeconds,
                metrics.firstCheckInSeconds, metrics.lastCheckOutSeconds, metrics.isWorking,
                metrics.requiredWorkSeconds, now))
        } catch (error: Exception) {
            if (error is kotlinx.coroutines.CancellationException) throw error
            publish(current().copy(loading = false, stale = true, error = error.message ?: "Connection failed"))
        }
        return current()
    }

    fun timeline(mode: TimelineMode): TimelineDto? {
        val current = current()
        val day = current.day ?: return null
        val user = account.user.value ?: return null
        return gateway.buildTimeline(TimelineInput(DayCalculationInput(day, user.timeZone,
            current.sampleEpoch, current.summary?.workTargetSeconds ?: day.requiredWorkSeconds,
            current.summary?.beforeStartDate ?: false), mode))
    }

    fun tick() {
        val (sample, anchor) = synchronized(stateLock) { _state.value to sampleElapsedRealtimeMillis }
        val day = sample.day ?: return
        val user = account.user.value ?: return
        val elapsed = SystemClock.elapsedRealtime()
        val now = if (sample.sampleEpoch > 0 && anchor != null) {
            sample.sampleEpoch + ((elapsed - anchor) / 1000L).coerceAtLeast(0)
        } else {
            Instant.now().epochSecond
        }
        runCatching {
            gateway.calculateDay(DayCalculationInput(day, user.timeZone, now,
                sample.summary?.workTargetSeconds ?: day.requiredWorkSeconds,
                sample.summary?.beforeStartDate ?: false))
        }.onSuccess { metrics ->
            synchronized(stateLock) {
                val latest = _state.value
                if (latest.day?.date == day.date && latest.sampleEpoch <= sample.sampleEpoch) {
                    sampleElapsedRealtimeMillis = elapsed
                    _state.value = latest.copy(metrics = metrics, sampleEpoch = now)
                }
            }
        }
    }

    suspend fun perform(checkIn: Boolean) = refreshMutex.withLock {
        try {
            actions.execute(checkIn)
        } finally {
            refreshTodayLocked()
        }
    }

    suspend fun cached(): DaySnapshotEntity? {
        val latest = cache.latest() ?: return null
        val zone = account.user.value?.timeZone ?: return latest
        val today = LocalDate.now(ZoneId.of(zone)).toString()
        return latest.takeIf { it.date == today }
    }

    fun clear() {
        synchronized(stateLock) {
            sampleElapsedRealtimeMillis = null
            _state.value = WorkState()
        }
    }

    private fun current(): WorkState = synchronized(stateLock) { _state.value }

    private fun publish(value: WorkState) {
        synchronized(stateLock) { _state.value = value }
    }
}
