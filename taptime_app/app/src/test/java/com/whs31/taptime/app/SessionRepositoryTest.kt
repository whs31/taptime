package com.whs31.taptime.app

import com.whs31.taptime.app.data.*
import io.mockk.*
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test

class SessionRepositoryTest {
    @Test fun missing_token_does_not_call_remote_user_api() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        val store = mockk<TokenStore>()
        val dao = mockk<SnapshotDao>()
        every { store.read() } returns null
        val repository = SessionRepository(gateway, store, dao)
        assertFalse(repository.restore())
        coVerify(exactly = 0) { gateway.fetchUser() }
    }
}
