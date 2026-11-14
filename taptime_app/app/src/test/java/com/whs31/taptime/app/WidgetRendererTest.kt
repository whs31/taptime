package com.whs31.taptime.app

import androidx.test.core.app.ApplicationProvider
import android.content.Context
import android.app.Application
import android.view.View
import android.widget.FrameLayout
import org.robolectric.annotation.Config
import com.whs31.taptime.app.widgets.*
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
@Config(application = Application::class, sdk = [31])
class WidgetRendererTest {
    @Test fun all_seven_widgets_render_logged_out_state() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val renderer = WidgetRenderer(context)
        assertEquals(7, WidgetKind.entries.size)
        WidgetKind.entries.forEach { kind ->
            val views = renderer.render(31, kind, WidgetState(false, null, null, null))
            val root = views.apply(context, FrameLayout(context))
            assertNotNull(root.findViewById<View>(R.id.widget_value))
        }
    }
    @Test fun only_dashboard_and_compact_have_work_actions() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val renderer = WidgetRenderer(context)
        WidgetKind.entries.forEach { kind ->
            val root = renderer.render(32, kind, WidgetState(false, null, null, null))
                .apply(context, FrameLayout(context))
            val actionsExpected = kind == WidgetKind.Dashboard || kind == WidgetKind.Compact
            assertEquals(kind.name, actionsExpected, root.findViewById<View>(R.id.widget_check_in) != null)
            assertEquals(kind.name, actionsExpected, root.findViewById<View>(R.id.widget_check_out) != null)
        }
    }
}
