package com.whs31.taptime.app.ui.today

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.data.WorkRepository
import com.whs31.taptime.app.domain.WorkClockTicker
import com.whs31.taptime.app.rust.*
import com.whs31.taptime.app.domain.TimelineMode as UiTimelineMode
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class TodayViewModel(private val app: AppContainer) : ViewModel() {
    val today = app.work.state
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    private val _message = MutableStateFlow<String?>(null)
    val message = _message.asStateFlow()
    private val _timeline = MutableStateFlow<TimelineDto?>(null)
    val timeline = _timeline.asStateFlow()
    private val _mode = MutableStateFlow(app.preferences.timelineMode)
    val mode = _mode.asStateFlow()
    fun selectMode(mode: UiTimelineMode) {
        app.preferences.timelineMode = mode
        _mode.value = mode
        updateTimeline()
    }
    private fun updateTimeline() {
        _timeline.value = runCatching {
            app.work.timeline(if (_mode.value == UiTimelineMode.WORK_DAY)
                TimelineMode.WORK_DAY else TimelineMode.CALENDAR_DAY)
        }.getOrNull()
    }

    init {
        viewModelScope.launch { app.sync.refresh(); updateTimeline() }
        viewModelScope.launch { WorkClockTicker().minutes().collect { app.work.tick(); updateTimeline() } }
        viewModelScope.launch { today.collect { updateTimeline() } }
    }
    fun refresh() { viewModelScope.launch { app.sync.refresh() } }
    fun action(checkIn: Boolean) { execute { app.work.perform(checkIn) } }
    fun add(date: String, seconds: Long, checkIn: Boolean) { execute { app.history.addManualEvent(date, seconds, checkIn) } }
    fun delete(id: String) { execute { app.history.deleteEvent(id) } }
    fun toggle(date: String, flag: DayFlag) { execute { app.history.toggleFlag(date, flag) } }
    fun setOverride(date: String, seconds: Long?) { execute { app.history.setOverride(date, seconds) } }
    private fun execute(block: suspend () -> Unit) {
        if (_busy.value) return
        viewModelScope.launch {
            _busy.value = true; _message.value = null
            try { block(); app.widgets.updateAll() }
            catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _message.value = e.message ?: "Ошибка запроса" }
            finally { _busy.value = false }
        }
    }
}
