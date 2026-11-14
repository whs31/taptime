package com.whs31.taptime.app.ui.history

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.time.LocalDate

class HistoryViewModel(private val app: AppContainer) : ViewModel() {
    private val _date = MutableStateFlow(LocalDate.now(java.time.ZoneId.of(app.account.timeZone)).toString())
    val date = _date.asStateFlow()
    private val _day = MutableStateFlow<DayDto?>(null)
    val day = _day.asStateFlow()
    private val _error = MutableStateFlow<String?>(null)
    val error = _error.asStateFlow()
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    init { load(_date.value) }
    fun load(date: String) {
        runCatching { LocalDate.parse(date) }.onFailure { _error.value = "Неверная дата"; return }
        _date.value = date
        viewModelScope.launch { _busy.value = true
            try { _day.value = app.history.day(date); _error.value = null }
            catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _error.value = e.message; _day.value = null }
            finally { _busy.value = false }
        }
    }
    fun add(seconds: Long, checkIn: Boolean) = mutate { app.history.addManualEvent(_date.value, seconds, checkIn) }
    fun delete(id: String) = mutate { app.history.deleteEvent(id) }
    fun toggle(flag: DayFlag) = mutate { app.history.toggleFlag(_date.value, flag) }
    fun overrideHours(value: Long?) = mutate { app.history.setOverride(_date.value, value) }
    private fun mutate(block: suspend () -> Unit) {
        if (_busy.value) return
        viewModelScope.launch { _busy.value = true
            try { block(); _day.value = app.history.day(_date.value); _error.value = null }
            catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _error.value = e.message }
            finally { _busy.value = false }
        }
    }
}
