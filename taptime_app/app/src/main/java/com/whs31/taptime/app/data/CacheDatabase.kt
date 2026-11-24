package com.whs31.taptime.app.data

import androidx.room.Database
import androidx.room.RoomDatabase

@Database(entities = [DaySnapshotEntity::class], version = 1, exportSchema = false)
abstract class CacheDatabase : RoomDatabase() {
    abstract fun snapshotDao(): SnapshotDao
}
