package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*

class RustTapTimeGateway(endpoint: String, private val onSessionExpired: suspend () -> Unit = {}) : TapTimeGateway {
    private val sdk = TapTimeSdk(endpoint)
    private suspend fun <T> request(block: suspend () -> T): T {
        try { return block() }
        catch (error: SdkException.Failure) {
            if (error.code == SdkErrorCode.UNAUTHENTICATED) onSessionExpired()
            throw error
        }
    }
    override suspend fun login(email: String, password: String) = request { sdk.login(email, password) }
    override suspend fun register(input: RegisterInput) = request { sdk.register(input) }
    override fun restoreToken(token: String) = sdk.setSessionToken(token)
    override fun clearToken() = sdk.clearSessionToken()
    override suspend fun fetchUser() = request { sdk.fetchUser() }
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
}
