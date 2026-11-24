package com.whs31.taptime.app.ui.reports

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.rust.ReportDto
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.time.LocalDate

class ReportsViewModel(private val app: AppContainer) : ViewModel() {
    private val _loading = MutableStateFlow(false)
    val loading = _loading.asStateFlow()
    private val _error = MutableStateFlow<String?>(null)
    val error = _error.asStateFlow()
    val report = app.reports.report
    private val _range = MutableStateFlow("Месяц")
    val range = _range.asStateFlow()
    init { select("Месяц") }
    fun select(value: String) {
        _range.value = value
        viewModelScope.launch {
            _loading.value = true; _error.value = null
            try {
                val now = LocalDate.now(java.time.ZoneId.of(app.account.timeZone))
                val start = when(value) {
                    "Прошлый месяц" -> now.minusMonths(1).withDayOfMonth(1)
                    "Квартал" -> now.withMonth((now.monthValue - 1) / 3 * 3 + 1).withDayOfMonth(1)
                    "Год" -> now.withDayOfYear(1)
                    else -> now.withDayOfMonth(1)
                }
                val end = if (value == "Прошлый месяц") start.plusMonths(1).minusDays(1) else now
                app.reports.refresh(start, end)
            } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _error.value = e.message }
            finally { _loading.value = false }
        }
    }
}
