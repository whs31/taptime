package com.whs31.taptime.app

import androidx.test.core.app.ApplicationProvider
import android.content.Context
import com.whs31.taptime.app.domain.TimelineMode
import com.whs31.taptime.app.widgets.WidgetPreferences
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class WidgetPreferencesTest {
    @Test fun each_widget_keeps_its_own_timeline_mode() {
        val prefs = WidgetPreferences(ApplicationProvider.getApplicationContext<Context>())
        prefs.setMode(100, TimelineMode.CALENDAR_DAY)
        prefs.setMode(101, TimelineMode.WORK_DAY)
        assertEquals(TimelineMode.CALENDAR_DAY, prefs.mode(100, TimelineMode.WORK_DAY))
        assertEquals(TimelineMode.WORK_DAY, prefs.mode(101, TimelineMode.CALENDAR_DAY))
        prefs.delete(100)
        prefs.delete(101)
    }
}
