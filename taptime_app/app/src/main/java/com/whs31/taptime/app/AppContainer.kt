package com.whs31.taptime.app

import android.app.Application
import androidx.room.Room
import com.whs31.taptime.app.data.*
import com.whs31.taptime.app.domain.*
import com.whs31.taptime.app.sync.*
import com.whs31.taptime.app.widgets.WidgetUpdateCoordinator
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class AppContainer(private val app: Application) {
    val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    val gateway: TapTimeGateway by lazy { RustTapTimeGateway("https://api.time.whs31.com", onSessionExpired = { logout() }) }
    val database: CacheDatabase by lazy { Room.databaseBuilder(app, CacheDatabase::class.java, "taptime-cache").build() }
    val tokenStore: TokenStore by lazy { KeystoreTokenStore(app) }
    val preferences: AppPreferences by lazy { AppPreferences(app) }
    val session by lazy { SessionRepository(gateway, tokenStore, database.snapshotDao()) }
    val account by lazy { AccountRepository(gateway) }
    val actions by lazy { WorkActionCoordinator(gateway, account) }
    val work by lazy { WorkRepository(gateway, account, actions, database.snapshotDao()) }
    val history by lazy { HistoryRepository(gateway, work, actions) }
    val reports by lazy { ReportRepository(gateway) }
    val power by lazy { PowerPolicy(app) }
    val widgets by lazy { WidgetUpdateCoordinator(app, this) }
    val sync by lazy { SyncCoordinator(account, work, reports, widgets) }
    val scheduler by lazy { SyncScheduler(app, power) }
    fun initialize() {
        scope.launch {
            if (session.restore()) {
                runCatching { account.refresh() }
                runCatching { sync.refresh() }
                scheduler.start()
            } else widgets.updateAll()
        }
    }
    suspend fun logout() {
        session.logout()
        account.clear()
        work.clear()
        reports.clear()
        scheduler.stop()
        widgets.updateAll()
    }
}
