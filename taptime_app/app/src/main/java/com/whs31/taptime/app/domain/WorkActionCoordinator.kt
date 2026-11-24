package com.whs31.taptime.app.domain

import com.whs31.taptime.app.data.AccountRepository
import com.whs31.taptime.app.data.TapTimeGateway
import com.whs31.taptime.app.rust.EventInput
import com.whs31.taptime.app.rust.EventKind
import kotlinx.coroutines.sync.Mutex
import java.time.Instant
import java.time.ZoneId

class WorkActionCoordinator(private val gateway: TapTimeGateway, private val account: AccountRepository) {
    private val mutex = Mutex()
    suspend fun <T> mutate(action: suspend () -> T): T {
        check(mutex.tryLock()) { "Операция уже выполняется" }
        try { return action() } finally { mutex.unlock() }
    }
    suspend fun execute(checkIn: Boolean) = mutate {
        val now = Instant.now().atZone(ZoneId.of(account.timeZone))
        val date = now.toLocalDate().toString()
        val recent = gateway.fetchDay(date).events.lastOrNull()?.kind
        if (checkIn && recent == EventKind.CHECK_IN) error("Check In уже выполнен")
        if (!checkIn && recent != EventKind.CHECK_IN) error("Нет открытой рабочей сессии")
        val input = EventInput(date, now.toLocalTime().toSecondOfDay().toLong())
        if (checkIn) gateway.checkIn(input) else gateway.checkOut(input)
    }
}
