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
}
