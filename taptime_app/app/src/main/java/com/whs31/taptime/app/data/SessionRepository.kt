package com.whs31.taptime.app.data

import android.util.Log
import com.whs31.taptime.app.rust.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class SessionRepository(private val gateway: TapTimeGateway, private val tokens: TokenStore, private val cache: SnapshotDao) {
    private val _authenticated = MutableStateFlow(false)
    private val mutex = Mutex()
    val authenticated = _authenticated.asStateFlow()

    suspend fun restore(): Boolean = mutex.withLock {
        val token = try {
            tokens.read()
        } catch (error: Exception) {
            Log.w(TAG, "restore token_read_failed type=${error.javaClass.simpleName}")
            gateway.clearToken()
            _authenticated.value = false
            return@withLock false
        } ?: run {
            Log.i(TAG, "restore token_absent")
            gateway.clearToken()
            _authenticated.value = false
            return@withLock false
        }
        try {
            gateway.restoreToken(token)
            _authenticated.value = true
            Log.i(TAG, "restore token_loaded")
        } catch (error: Exception) {
            Log.w(TAG, "restore sdk_token_failed type=${error.javaClass.simpleName}")
            _authenticated.value = false
            return@withLock false
        }
        try {
            gateway.fetchUser()
            Log.i(TAG, "restore server_validated")
            true
        } catch (error: SdkException.Failure) {
            if (isConfirmedServerExpiry(error)) {
                Log.w(TAG, "restore server_rejected_session")
                clearConfirmedSession()
                false
            } else {
                Log.w(TAG, "restore retained_offline code=${error.code}")
                true
            }
        } catch (error: Exception) {
            Log.w(TAG, "restore retained_after_error type=${error.javaClass.simpleName}")
            true
        }
    }

    suspend fun login(email: String, password: String): AuthResult = mutex.withLock {
        val response = gateway.login(email, password)
        persistLogin(response)
        Log.i(TAG, "login saved_session")
        response
    }

    suspend fun register(input: RegisterInput): AuthResult = mutex.withLock {
        val response = gateway.register(input)
        persistLogin(response)
        Log.i(TAG, "register saved_session")
        response
    }

    suspend fun logout() = mutex.withLock {
        Log.i(TAG, "logout explicit")
        gateway.clearToken()
        tokens.clear()
        cache.clear()
        _authenticated.value = false
    }

    private fun persistLogin(response: AuthResult) {
        try {
            tokens.write(response.token)
        } catch (error: Exception) {
            Log.w(TAG, "login token_write_failed type=${error.javaClass.simpleName}")
            gateway.clearToken()
            _authenticated.value = false
            throw error
        }
        _authenticated.value = true
    }

    private suspend fun clearConfirmedSession() {
        gateway.clearToken()
        tokens.clear()
        cache.clear()
        _authenticated.value = false
    }

    private fun isConfirmedServerExpiry(error: SdkException.Failure): Boolean =
        error.code == SdkErrorCode.UNAUTHENTICATED && error.message == SERVER_EXPIRY_MESSAGE

    private companion object {
        const val TAG = "TapTimeSession"
        const val SERVER_EXPIRY_MESSAGE = "Session expired"
    }
}
