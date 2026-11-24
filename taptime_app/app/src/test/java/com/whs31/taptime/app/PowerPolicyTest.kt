package com.whs31.taptime.app

import androidx.test.core.app.ApplicationProvider
import android.content.Context
import com.whs31.taptime.app.sync.PowerPolicy
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class PowerPolicyTest {
    @Test fun background_interval_respects_workmanager_minimum() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        assertTrue(PowerPolicy(context).backgroundMinutes() >= 15)
        assertTrue(PowerPolicy(context).foregroundMinutes() > 0)
    }
}
