package com.whs31.taptime.app

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import android.view.LayoutInflater
import android.widget.LinearLayout
import com.whs31.taptime.app.widgets.WidgetKind
import com.whs31.taptime.app.widgets.WidgetRenderer
import com.whs31.taptime.app.widgets.WidgetState
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class WidgetIntegrationTest {
    @Test fun seven_widget_layouts_have_all_shared_remoteviews_ids() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            val inflater = LayoutInflater.from(context)
            WidgetKind.entries.forEach { kind ->
                val widget = inflater.inflate(kind.layoutId, LinearLayout(context), false)
                assertNotNull(widget.findViewById<android.view.View>(R.id.widget_root))
                assertNotNull(widget.findViewById<android.view.View>(R.id.widget_chart))
                assertNotNull(widget.findViewById<android.view.View>(R.id.widget_clock))
            }
        }
    }
    @Test fun seven_remoteviews_render_without_private_data() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            WidgetKind.entries.forEachIndexed { index, kind ->
            val rendered = WidgetRenderer(context).render(200 + index, kind, WidgetState(false, null, null, null))
                .apply(context, LinearLayout(context))
            assertNotNull(rendered.findViewById<android.view.View>(R.id.widget_chart))
            if (kind == WidgetKind.MonthBalance || kind == WidgetKind.MonthRing) {
                assertNull(rendered.findViewById<android.view.View>(R.id.widget_check_in))
                assertNull(rendered.findViewById<android.view.View>(R.id.widget_check_out))
            }
            }
        }
    }

}
