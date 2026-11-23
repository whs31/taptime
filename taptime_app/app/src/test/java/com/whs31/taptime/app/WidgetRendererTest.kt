package com.whs31.taptime.app

import android.app.Application
import android.content.Context
import android.view.View
import android.widget.Chronometer
import android.widget.FrameLayout
import android.widget.TextView
import androidx.test.core.app.ApplicationProvider
import com.whs31.taptime.app.data.DaySnapshotEntity
import com.whs31.taptime.app.widgets.*
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

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

    @Test fun active_daily_widgets_use_the_same_live_work_counter() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val snapshot = DaySnapshotEntity(
            "2026-11-18",
            9 * 3600 + 60 + 33,
            20 * 60,
            20 * 60,
            8 * 3600,
            null,
            true,
            8 * 3600,
            1_794_992_493,
        )
        val renderer = WidgetRenderer(context)
        WidgetKind.entries.forEach { kind ->
            val root = renderer.render(33, kind, WidgetState(true, snapshot, null, null, stale = false))
                .apply(context, FrameLayout(context))
            val clock = root.findViewById<Chronometer>(R.id.widget_clock)
            val usesLiveCounter = kind in setOf(
                WidgetKind.Dashboard,
                WidgetKind.DayTimeline,
                WidgetKind.Compact,
                WidgetKind.Small,
                WidgetKind.Tiny,
            )
            assertEquals(kind.name, usesLiveCounter, clock.visibility == View.VISIBLE)
        }
    }

    @Test fun stopped_widget_values_preserve_seconds() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val snapshot = DaySnapshotEntity(
            "2026-11-18",
            9 * 3600 + 60 + 33,
            20 * 60,
            20 * 60,
            8 * 3600,
            17 * 3600,
            false,
            8 * 3600,
            1_794_992_493,
        )
        val root = WidgetRenderer(context).render(
            34,
            WidgetKind.Tiny,
            WidgetState(true, snapshot, null, null, stale = false),
        ).apply(context, FrameLayout(context))
        assertEquals("09:01:33", root.findViewById<TextView>(R.id.widget_value).text)
    }
}
