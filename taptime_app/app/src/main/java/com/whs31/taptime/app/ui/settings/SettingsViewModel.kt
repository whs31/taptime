package com.whs31.taptime.app.ui.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class SettingsViewModel(private val app: AppContainer) : ViewModel() {
    val user = app.account.user
    val preferences = app.preferences
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    private val _message = MutableStateFlow<String?>(null)
    val message = _message.asStateFlow()
    fun save(zone: String, workHours: Long, lunchMinutes: Long, weekends: List<Weekday>, remote: List<Weekday>, startDate: String?) {
        if (_busy.value) return
        viewModelScope.launch {
            _busy.value = true
            try {
                require(workHours in 0..24 && lunchMinutes in 0..1440)
                app.account.updateSettings(SettingsInput(zone, UserSettingsDto(workHours * 3600, lunchMinutes * 60, weekends, remote, startDate)))
                app.sync.refresh()
                _message.value = "Сохранено"
            } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _message.value = e.message ?: "Не удалось сохранить" }
            finally { _busy.value = false }
        }
    }
}
