package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.time.LocalDate

class ReportRepository(private val gateway: TapTimeGateway) {
    private val _report = MutableStateFlow<ReportDto?>(null)
    val report = _report.asStateFlow()
    suspend fun refresh(start: LocalDate, end: LocalDate): ReportDto {
        require(!start.isAfter(end))
        val dashboard = gateway.fetchDashboard(DashboardQuery(start.toString(), end.toString(),
            start.withDayOfMonth(1).toString(), end.toString(),
            LocalDate.now(java.time.ZoneId.of(gateway.fetchUser().timeZone)).toString()))
        return gateway.buildReport(ReportInput(start.toString(), end.toString(), dashboard.days)).also { _report.value = it }
    }
    fun clear() { _report.value = null }
}
