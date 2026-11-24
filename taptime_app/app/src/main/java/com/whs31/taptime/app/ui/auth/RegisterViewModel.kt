package com.whs31.taptime.app.ui.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.time.ZoneId

class RegisterViewModel(private val container: AppContainer) : ViewModel() {
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    private val _error = MutableStateFlow<String?>(null)
    val error = _error.asStateFlow()
    fun register(name: String, email: String, password: String) {
        if (_busy.value) return
        viewModelScope.launch {
            _busy.value = true
            try {
                val settings = UserSettingsDto(8*3600L, 1800L, listOf(Weekday.SATURDAY, Weekday.SUNDAY), emptyList(), null)
                container.session.register(RegisterInput(name, email, password, null, ZoneId.systemDefault().id, settings))
                container.account.refresh()
                container.sync.refresh()
                container.scheduler.start()
            } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _error.value = e.message ?: "Ошибка регистрации" }
            finally { _busy.value = false }
        }
    }
}
