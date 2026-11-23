package com.whs31.taptime.app.sync

import com.whs31.taptime.app.data.AccountRepository
import com.whs31.taptime.app.data.WorkRepository
import com.whs31.taptime.app.data.ReportRepository
import com.whs31.taptime.app.widgets.WidgetUpdateCoordinator
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class SyncCoordinator(private val account: AccountRepository, private val work: WorkRepository,
                      private val reports: ReportRepository, private val widgets: WidgetUpdateCoordinator) {
    private val mutex = Mutex()

    suspend fun refresh() = mutex.withLock {
        if (account.user.value == null) account.refresh()
        val result = work.refreshToday()
        widgets.updateAll()
        result
    }
}
