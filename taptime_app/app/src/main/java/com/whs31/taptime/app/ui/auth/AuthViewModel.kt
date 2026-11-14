package com.whs31.taptime.app.ui.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class AuthViewModel(private val container: AppContainer) : ViewModel() {
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    private val _error = MutableStateFlow<String?>(null)
    val error = _error.asStateFlow()
    fun signIn(email: String, password: String) {
        if (_busy.value) return
        viewModelScope.launch {
            _busy.value = true
            _error.value = null
            try {
                container.session.login(email, password)
                container.account.refresh()
                container.sync.refresh()
                container.scheduler.start()
            } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _error.value = e.message ?: "Ошибка авторизации" }
            finally { _busy.value = false }
        }
    }
}
