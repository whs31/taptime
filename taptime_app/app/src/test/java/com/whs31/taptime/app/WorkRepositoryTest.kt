package com.whs31.taptime.app

import com.whs31.taptime.app.data.*
import com.whs31.taptime.app.domain.WorkActionCoordinator
import io.mockk.*
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test

class WorkRepositoryTest {
    @Test fun network_failure_marks_snapshot_stale() = runBlocking {
        val gateway = mockk<TapTimeGateway>()
        coEvery { gateway.fetchUser() } throws java.io.IOException("offline")
        val account = AccountRepository(gateway)
        val repo = WorkRepository(gateway, account, WorkActionCoordinator(gateway, account), mockk())
        val result = repo.refreshToday()
        assertTrue(result.stale)
        assertNotNull(result.error)
        assertFalse(result.loading)
    }

    @Test fun cached_snapshot_is_available_before_account_refresh() = runBlocking {
        val gateway = mockk<TapTimeGateway>()
        val account = AccountRepository(gateway)
        val cache = mockk<SnapshotDao>()
        val snapshot = DaySnapshotEntity(
            "2026-11-18",
            9 * 3600,
            0,
            0,
            8 * 3600,
            null,
            true,
            8 * 3600,
            1_794_992_400,
        )
        coEvery { cache.latest() } returns snapshot
        val repo = WorkRepository(gateway, account, WorkActionCoordinator(gateway, account), cache)

        assertEquals(snapshot, repo.cached())
    }
}
