package com.whs31.taptime.app

import com.whs31.taptime.app.data.*
import com.whs31.taptime.app.domain.WorkActionCoordinator
import com.whs31.taptime.app.rust.*
import io.mockk.*
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test

class WorkActionCoordinatorTest {
    @Test fun refuses_duplicate_check_in_before_writing() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        val account = mockk<AccountRepository>()
        every { account.timeZone } returns "UTC"
        coEvery { gateway.fetchDay(any()) } returns DayDto("2026-11-14", listOf(EventDto("id", EventKind.CHECK_IN, 36000)), emptyList(), 28800, 1800)
        val coordinator = WorkActionCoordinator(gateway, account)
        val result = runCatching { coordinator.execute(true) }
        assertTrue(result.isFailure)
        coVerify(exactly = 0) { gateway.checkIn(any()) }
    }
    @Test fun check_out_requires_open_session() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        val account = mockk<AccountRepository>()
        every { account.timeZone } returns "UTC"
        coEvery { gateway.fetchDay(any()) } returns DayDto("2026-11-14", emptyList(), emptyList(), 28800, 1800)
        assertTrue(runCatching { WorkActionCoordinator(gateway, account).execute(false) }.isFailure)
        coVerify(exactly = 0) { gateway.checkOut(any()) }
    }
}
