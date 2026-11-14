package com.whs31.taptime.app.data

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
    suspend fun refreshToday(): WorkState = refreshMutex.withLock {
        _state.value = _state.value.copy(loading = true)
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
            _state.value = WorkState(metrics, dashboard, summary, day, monthly.balanceSeconds, now, loading = false, stale = false)
            cache.upsert(DaySnapshotEntity(today.toString(), metrics.workedSeconds, metrics.balanceSeconds,
                monthly.balanceSeconds,
                metrics.firstCheckInSeconds, metrics.lastCheckOutSeconds, metrics.isWorking,
                metrics.requiredWorkSeconds, now))
        } catch (error: Exception) {
            if (error is kotlinx.coroutines.CancellationException) throw error
            _state.value = _state.value.copy(loading = false, stale = true, error = error.message ?: "Connection failed")
        }
        _state.value
    }
    fun timeline(mode: TimelineMode): TimelineDto? {
        val current = state.value
        val day = current.day ?: return null
        val user = account.user.value ?: return null
        return gateway.buildTimeline(TimelineInput(DayCalculationInput(day, user.timeZone,
            Instant.now().epochSecond, current.summary?.workTargetSeconds ?: day.requiredWorkSeconds,
            current.summary?.beforeStartDate ?: false), mode))
    }
    fun tick() {
        val current = _state.value
        val day = current.day ?: return
        val user = account.user.value ?: return
        val now = Instant.now().epochSecond
        runCatching {
            gateway.calculateDay(DayCalculationInput(day, user.timeZone, now,
                current.summary?.workTargetSeconds ?: day.requiredWorkSeconds,
                current.summary?.beforeStartDate ?: false))
        }.onSuccess { _state.value = current.copy(metrics = it, sampleEpoch = now) }
    }
    suspend fun perform(checkIn: Boolean) {
        try { actions.execute(checkIn) } finally { refreshToday() }
    }
    suspend fun cached(): DaySnapshotEntity? {
        val user = account.user.value ?: return null
        val today = LocalDate.now(ZoneId.of(user.timeZone)).toString()
        return cache.latest()?.takeIf { it.date == today }
    }
    fun clear() { _state.value = WorkState() }
}
