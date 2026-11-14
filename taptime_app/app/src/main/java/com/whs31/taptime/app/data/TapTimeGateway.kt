package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*

interface TapTimeGateway {
    suspend fun login(email: String, password: String): AuthResult
    suspend fun register(input: RegisterInput): AuthResult
    fun restoreToken(token: String)
    fun clearToken()
    suspend fun fetchUser(): UserDto
    suspend fun fetchDay(date: String): DayDto
    suspend fun fetchDashboard(query: DashboardQuery): DashboardDto
    suspend fun checkIn(input: EventInput)
    suspend fun checkOut(input: EventInput)
    suspend fun deleteEvent(id: String)
    suspend fun toggleDayFlag(input: FlagInput)
    suspend fun setWorkOverride(input: WorkOverrideInput)
    suspend fun updateProfile(input: ProfileInput): UserDto
    suspend fun updateSettings(input: SettingsInput): UserDto
    suspend fun updateRfid(uid: String?): UserDto
    suspend fun deleteTimeData()
    suspend fun deleteAccount(password: String)
    fun calculateDay(input: DayCalculationInput): DayMetricsDto
    fun buildTimeline(input: TimelineInput): TimelineDto
    fun buildReport(input: ReportInput): ReportDto
}
