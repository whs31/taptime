package com.whs31.taptime.app.domain

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import java.time.Instant

class WorkClockTicker {
    fun minutes(): Flow<Long> = flow {
        while (true) {
            emit(Instant.now().epochSecond)
            val millis = System.currentTimeMillis()
            delay(60_000 - millis % 60_000)
        }
    }
}
