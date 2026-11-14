package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class SessionRepository(private val gateway: TapTimeGateway, private val tokens: TokenStore, private val cache: SnapshotDao) {
    private val _authenticated = MutableStateFlow(false)
    val authenticated = _authenticated.asStateFlow()
    suspend fun restore(): Boolean {
        val token = tokens.read() ?: return false
        return try {
            gateway.restoreToken(token)
            gateway.fetchUser()
            _authenticated.value = true
            true
        } catch (_: Exception) {
            gateway.clearToken()
            _authenticated.value = false
            false
        }
    }
    suspend fun login(email: String, password: String): AuthResult {
        val response = gateway.login(email, password)
        tokens.write(response.token)
        _authenticated.value = true
        return response
    }
    suspend fun register(input: RegisterInput): AuthResult {
        val response = gateway.register(input)
        tokens.write(response.token)
        _authenticated.value = true
        return response
    }
    suspend fun logout() {
        gateway.clearToken()
        tokens.clear()
        cache.clear()
        _authenticated.value = false
    }
}
