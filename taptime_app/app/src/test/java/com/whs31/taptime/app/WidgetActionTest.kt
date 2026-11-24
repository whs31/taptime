package com.whs31.taptime.app

import android.content.Intent
import androidx.test.core.app.ApplicationProvider
import android.content.Context
import com.whs31.taptime.app.widgets.WidgetActionReceiver
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class WidgetActionTest {
    @Test fun receiver_ignores_unrelated_and_invalid_actions() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val receiver = WidgetActionReceiver()
        receiver.onReceive(context, Intent("not-a-widget-action"))
        receiver.onReceive(context, Intent(WidgetActionReceiver.ACTION))
    }
}
