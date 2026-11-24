package com.whs31.taptime.app.data

import androidx.room.Entity
import androidx.room.PrimaryKey

object CacheEntities { const val DAY_SNAPSHOTS = "day_snapshots" }

@Entity(tableName = CacheEntities.DAY_SNAPSHOTS)
data class DaySnapshotEntity(
    @PrimaryKey val date: String,
    val workedSeconds: Long,
    val balanceSeconds: Long,
    val monthlyBalanceSeconds: Long,
    val firstCheckInSeconds: Long?,
    val lastCheckOutSeconds: Long?,
    val isWorking: Boolean,
    val requiredWorkSeconds: Long,
    val sampledAtEpochSeconds: Long,
)
