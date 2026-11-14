package com.whs31.taptime.app.ui.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.rust.ProfileInput
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class ProfileViewModel(private val app: AppContainer) : ViewModel() {
    val user = app.account.user
    private val _busy = MutableStateFlow(false)
    val busy = _busy.asStateFlow()
    private val _message = MutableStateFlow<String?>(null)
    val message = _message.asStateFlow()
    fun save(name: String, email: String, organization: String, rfid: String?) = run {
        execute {
            app.account.updateProfile(ProfileInput(name, email, organization.ifBlank { null }))
            app.account.updateRfid(rfid)
        }
    }
    fun logout() = execute { app.logout() }
    fun deleteTimeData() = execute { app.account.deleteTimeData(); app.work.refreshToday(); app.widgets.updateAll() }
    fun deleteAccount(password: String) = execute { app.account.deleteAccount(password); app.logout() }
    private fun execute(block: suspend () -> Unit) {
        if (_busy.value) return
        viewModelScope.launch { _busy.value = true
            try { block(); _message.value = "Готово" }
            catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            _message.value = e.message ?: "Ошибка" }
            finally { _busy.value = false }
        }
    }
}
