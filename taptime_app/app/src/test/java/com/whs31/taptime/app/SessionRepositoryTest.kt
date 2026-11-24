package com.whs31.taptime.app

import com.whs31.taptime.app.data.*
import com.whs31.taptime.app.rust.AuthResult
import com.whs31.taptime.app.rust.SdkErrorCode
import com.whs31.taptime.app.rust.SdkException
import com.whs31.taptime.app.rust.UserDto
import io.mockk.*
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [31])
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

    @Test fun login_then_process_restart_restores_session() = runBlocking {
        val store = MemoryTokenStore()
        val firstGateway = mockk<TapTimeGateway>(relaxed = true)
        coEvery { firstGateway.login(any(), any()) } returns AuthResult("saved-token", mockk<UserDto>())
        val dao = mockk<SnapshotDao>(relaxed = true)
        SessionRepository(firstGateway, store, dao).login("person@example.test", "password")

        val secondGateway = mockk<TapTimeGateway>(relaxed = true)
        val restored = SessionRepository(secondGateway, store, dao)
        assertTrue(restored.restore())
        assertTrue(restored.authenticated.value)
        verify { secondGateway.restoreToken("saved-token") }
        assertEquals(0, store.clearCount)
    }

    @Test fun offline_restore_keeps_saved_session() = assertRetainedFor(
        SdkException.Failure(SdkErrorCode.UNAVAILABLE, "Connection unavailable", true, false)
    )

    @Test fun server_error_restore_keeps_saved_session() = assertRetainedFor(
        SdkException.Failure(SdkErrorCode.INTERNAL, "TapTime request failed", false, false)
    )

    @Test fun timeout_restore_keeps_saved_session() = assertRetainedFor(
        SdkException.Failure(SdkErrorCode.TIMEOUT, "Request timed out; outcome may be unknown", true, false)
    )

    @Test fun concurrent_restore_serializes_network_validation() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        coEvery { gateway.fetchUser() } coAnswers { delay(30); mockk<UserDto>() }
        val repository = SessionRepository(gateway, MemoryTokenStore("saved-token"), mockk(relaxed = true))
        val results = coroutineScope {
            listOf(async { repository.restore() }, async { repository.restore() }).map { it.await() }
        }
        assertEquals(listOf(true, true), results)
        verify(exactly = 2) { gateway.restoreToken("saved-token") }
        coVerify(exactly = 2) { gateway.fetchUser() }
    }

    @Test fun confirmed_server_expiry_clears_session_and_cache() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        coEvery { gateway.fetchUser() } throws expiredError()
        val store = MemoryTokenStore("saved-token")
        val dao = mockk<SnapshotDao>(relaxed = true)
        val repository = SessionRepository(gateway, store, dao)
        assertFalse(repository.restore())
        assertFalse(repository.authenticated.value)
        assertEquals(1, store.clearCount)
        verify { gateway.clearToken() }
        coVerify { dao.clear() }
    }

    @Test fun local_unauthenticated_error_does_not_clear_saved_session() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        coEvery { gateway.fetchUser() } throws SdkException.Failure(
            SdkErrorCode.UNAUTHENTICATED, "Login required", false, false
        )
        val store = MemoryTokenStore("saved-token")
        val repository = SessionRepository(gateway, store, mockk(relaxed = true))
        assertTrue(repository.restore())
        assertTrue(repository.authenticated.value)
        assertEquals(0, store.clearCount)
    }

    @Test fun unreadable_keystore_does_not_silently_delete_record() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        val store = MemoryTokenStore(readFailure = IllegalStateException("key lost"))
        val repository = SessionRepository(gateway, store, mockk(relaxed = true))
        assertFalse(repository.restore())
        assertFalse(repository.authenticated.value)
        assertEquals(0, store.clearCount)
        verify { gateway.clearToken() }
    }

    @Test fun explicit_logout_removes_token_and_cache() = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        val store = MemoryTokenStore("saved-token")
        val dao = mockk<SnapshotDao>(relaxed = true)
        SessionRepository(gateway, store, dao).logout()
        assertEquals(1, store.clearCount)
        verify { gateway.clearToken() }
        coVerify { dao.clear() }
    }

    private fun assertRetainedFor(error: SdkException.Failure) = runBlocking {
        val gateway = mockk<TapTimeGateway>(relaxed = true)
        coEvery { gateway.fetchUser() } throws error
        val store = MemoryTokenStore("saved-token")
        val repository = SessionRepository(gateway, store, mockk(relaxed = true))
        assertTrue(repository.restore())
        assertTrue(repository.authenticated.value)
        assertEquals(0, store.clearCount)
    }

    private fun expiredError() = SdkException.Failure(
        SdkErrorCode.UNAUTHENTICATED, "Session expired", false, false
    )

    private class MemoryTokenStore(
        private var value: String? = null,
        private val readFailure: Exception? = null,
    ) : TokenStore {
        var clearCount = 0
        override fun read(): String? {
            readFailure?.let { throw it }
            return value
        }
        override fun write(token: String) { value = token }
        override fun clear() {
            clearCount += 1
            value = null
        }
    }
}
