package com.whs31.taptime.app.data

import com.whs31.taptime.app.rust.*
import com.whs31.taptime.app.domain.WorkActionCoordinator

class HistoryRepository(private val gateway: TapTimeGateway, private val work: WorkRepository, private val actions: WorkActionCoordinator) {
    suspend fun day(date: String) = gateway.fetchDay(date)
    suspend fun addManualEvent(date: String, seconds: Long, checkIn: Boolean) {
        val input = EventInput(date, seconds)
        try { actions.mutate { if (checkIn) gateway.checkIn(input) else gateway.checkOut(input) } }
        finally { work.refreshToday() }
    }
    suspend fun deleteEvent(id: String) { try { actions.mutate { gateway.deleteEvent(id) } } finally { work.refreshToday() } }
    suspend fun toggleFlag(date: String, flag: DayFlag) { try { actions.mutate { gateway.toggleDayFlag(FlagInput(date, flag)) } } finally { work.refreshToday() } }
    suspend fun setOverride(date: String, seconds: Long?) { try { actions.mutate { gateway.setWorkOverride(WorkOverrideInput(date, seconds)) } } finally { work.refreshToday() } }
}
