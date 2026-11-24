package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class AccountRepository(private val gateway: TapTimeGateway) {
    private val _user = MutableStateFlow<UserDto?>(null)
    val user = _user.asStateFlow()
    val timeZone: String get() = _user.value?.timeZone ?: throw IllegalStateException("User timezone unknown")
    suspend fun refresh(): UserDto = gateway.fetchUser().also { _user.value = it }
    suspend fun updateProfile(input: ProfileInput) = gateway.updateProfile(input).also { _user.value = it }
    suspend fun updateSettings(input: SettingsInput) = gateway.updateSettings(input).also { _user.value = it }
    suspend fun updateRfid(uid: String?) = gateway.updateRfid(uid).also { _user.value = it }
    suspend fun deleteTimeData() = gateway.deleteTimeData()
    suspend fun deleteAccount(password: String) = gateway.deleteAccount(password)
    fun clear() { _user.value = null }
}
