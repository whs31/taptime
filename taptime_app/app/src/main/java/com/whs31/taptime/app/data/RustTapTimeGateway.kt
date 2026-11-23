package com.whs31.taptime.app.data

import android.util.Log
import com.whs31.taptime.app.rust.*

class RustTapTimeGateway(endpoint: String, private val onSessionExpired: suspend () -> Unit = {}) : TapTimeGateway {
    private val sdk = TapTimeSdk(endpoint)
    private var pendingRestoreValidation = false
    private suspend fun <T> request(invalidateOnExpiry: Boolean = true, block: suspend () -> T): T {
        try { return block() }
        catch (error: SdkException.Failure) {
            if (invalidateOnExpiry && isConfirmedServerExpiry(error)) {
                Log.w(TAG, "server_rejected_active_session")
                onSessionExpired()
            }
            throw error
        }
    }
    override suspend fun login(email: String, password: String) = request { sdk.login(email, password) }
    override suspend fun register(input: RegisterInput) = request { sdk.register(input) }
    override fun restoreToken(token: String) {
        sdk.setSessionToken(token)
        synchronized(this) { pendingRestoreValidation = true }
    }
    override fun clearToken() {
        synchronized(this) { pendingRestoreValidation = false }
        sdk.clearSessionToken()
    }
    override suspend fun fetchUser() = request(!consumeRestoreValidation()) { sdk.fetchUser() }
    override suspend fun fetchDay(date: String) = request { sdk.fetchDay(date) }
    override suspend fun fetchDashboard(query: DashboardQuery) = request { sdk.fetchDashboard(query) }
    override suspend fun checkIn(input: EventInput) = request { sdk.checkIn(input) }
    override suspend fun checkOut(input: EventInput) = request { sdk.checkOut(input) }
    override suspend fun deleteEvent(id: String) = request { sdk.deleteEvent(id) }
    override suspend fun toggleDayFlag(input: FlagInput) = request { sdk.toggleDayFlag(input) }
    override suspend fun setWorkOverride(input: WorkOverrideInput) = request { sdk.setWorkOverride(input) }
    override suspend fun updateProfile(input: ProfileInput) = request { sdk.updateProfile(input) }
    override suspend fun updateSettings(input: SettingsInput) = request { sdk.updateSettings(input) }
    override suspend fun updateRfid(uid: String?) = request { sdk.updateRfid(uid) }
    override suspend fun deleteTimeData() = request { sdk.deleteTimeData() }
    override suspend fun deleteAccount(password: String) = request { sdk.deleteAccount(password) }
    override fun calculateDay(input: DayCalculationInput) = sdk.calculateDay(input)
    override fun buildTimeline(input: TimelineInput) = sdk.buildTimeline(input)
    override fun buildReport(input: ReportInput) = sdk.buildReport(input)

    private fun consumeRestoreValidation(): Boolean = synchronized(this) {
        pendingRestoreValidation.also { pendingRestoreValidation = false }
    }

    private fun isConfirmedServerExpiry(error: SdkException.Failure): Boolean =
        error.code == SdkErrorCode.UNAUTHENTICATED && error.message == SERVER_EXPIRY_MESSAGE

    private companion object {
        const val TAG = "TapTimeGateway"
        const val SERVER_EXPIRY_MESSAGE = "Session expired"
    }
}
