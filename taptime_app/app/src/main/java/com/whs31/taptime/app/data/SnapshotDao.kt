package com.whs31.taptime.app.data

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query

@Dao
interface SnapshotDao {
    @Query("SELECT * FROM day_snapshots ORDER BY date DESC LIMIT 1")
    suspend fun latest(): DaySnapshotEntity?
    @Query("SELECT * FROM day_snapshots WHERE date=:date LIMIT 1")
    suspend fun forDate(date: String): DaySnapshotEntity?
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(value: DaySnapshotEntity)
    @Query("DELETE FROM day_snapshots")
    suspend fun clear()
}
