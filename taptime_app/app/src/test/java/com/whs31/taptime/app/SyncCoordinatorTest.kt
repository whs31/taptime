package com.whs31.taptime.app

import com.whs31.taptime.app.data.*
import com.whs31.taptime.app.sync.SyncCoordinator
import com.whs31.taptime.app.widgets.WidgetUpdateCoordinator
import io.mockk.*
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test

class SyncCoordinatorTest {
    @Test fun refresh_does_not_query_month_when_no_widgets() = runBlocking {
        val account = mockk<AccountRepository>()
        val work = mockk<WorkRepository>()
        val reports = mockk<ReportRepository>()
        val widgets = mockk<WidgetUpdateCoordinator>()
        every { account.user.value } returns null
        coEvery { account.refresh() } returns mockk()
        coEvery { work.refreshToday() } returns WorkRepository.WorkState()
        coEvery { widgets.updateAll() } just Runs

        val output = SyncCoordinator(account, work, reports, widgets).refresh()

        assertTrue(output.stale)
        coVerify(exactly = 0) { reports.refresh(any(), any()) }
        coVerify(exactly = 1) { widgets.updateAll() }
    }

    @Test fun refresh_uses_work_snapshot_for_month_widgets() = runBlocking {
        val account = mockk<AccountRepository>()
        val work = mockk<WorkRepository>()
        val reports = mockk<ReportRepository>()
        val widgets = mockk<WidgetUpdateCoordinator>()
        every { account.user.value } returns null
        coEvery { account.refresh() } returns mockk()
        coEvery { work.refreshToday() } returns WorkRepository.WorkState(stale = false)
        coEvery { widgets.updateAll() } just Runs

        val output = SyncCoordinator(account, work, reports, widgets).refresh()

        assertFalse(output.stale)
        coVerify(exactly = 0) { reports.refresh(any(), any()) }
        coVerify(exactly = 1) { widgets.updateAll() }
    }
}
