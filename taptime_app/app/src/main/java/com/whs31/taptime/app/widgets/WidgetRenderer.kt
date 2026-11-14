package com.whs31.taptime.app.widgets

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import android.os.SystemClock
import android.os.Build
import android.content.res.Configuration
import android.content.res.ColorStateList
import android.util.TypedValue
import android.view.View
import android.widget.RemoteViews
import com.whs31.taptime.app.MainActivity
import com.whs31.taptime.app.R
import com.whs31.taptime.app.ui.components.formatSeconds

class WidgetRenderer(private val context: Context) {
    private val painter = WidgetChartPainter(context)
    fun render(widgetId: Int, kind: WidgetKind, state: WidgetState): RemoteViews {
        val view = RemoteViews(context.packageName, kind.layoutId)
        val dark = context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES
        val surface = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_neutral1_900 else android.R.color.system_neutral1_50) else if (dark) 0xff202124.toInt() else 0xfff7f8fa.toInt()
        val foreground = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_neutral1_50 else android.R.color.system_neutral1_900) else if (dark) 0xfff1f3f4.toInt() else 0xff202124.toInt()
        val secondary = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_neutral2_200 else android.R.color.system_neutral2_700) else if (dark) 0xffbdc1c6.toInt() else 0xff5f6368.toInt()
        val accent = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_accent1_200 else android.R.color.system_accent1_600) else if (dark) 0xffabc7ff.toInt() else 0xff315eaa.toInt()
        view.setInt(R.id.widget_root, "setBackgroundColor", surface)
        if (Build.VERSION.SDK_INT >= 31) view.setViewOutlinePreferredRadius(R.id.widget_root, 20f, TypedValue.COMPLEX_UNIT_DIP)
        view.setTextColor(R.id.widget_value, foreground)
        view.setTextColor(R.id.widget_clock, foreground)
        view.setTextColor(R.id.widget_title, secondary)
        view.setTextColor(R.id.widget_subtitle, secondary)
        view.setTextViewText(R.id.widget_title, context.getString(when (kind) {
            WidgetKind.DayTimeline -> R.string.widget_day
            WidgetKind.MonthBalance -> R.string.widget_month_balance
            WidgetKind.MonthRing -> R.string.widget_month_progress
            else -> R.string.app_name
        }))
        val snapshot = state.snapshot
        if (!state.authenticated) {
            view.setTextViewText(R.id.widget_value, context.getString(R.string.widget_sign_in))
            view.setTextViewText(R.id.widget_subtitle, context.getString(R.string.widget_open_app))
        } else if (snapshot == null) {
            view.setTextViewText(R.id.widget_value, "--:--")
            view.setTextViewText(R.id.widget_subtitle, context.getString(R.string.widget_no_data))
        } else {
            view.setTextViewText(R.id.widget_value, formatSeconds(snapshot.workedSeconds))
            val report = state.report
            view.setTextViewText(R.id.widget_subtitle, when (kind) {
                WidgetKind.Dashboard -> state.remainingSeconds?.let { context.getString(R.string.widget_remaining, formatSeconds(it)) } ?: context.getString(R.string.widget_no_data)
                WidgetKind.MonthBalance -> report?.let { "${it.rangeStart} — ${it.rangeEnd}" } ?: context.getString(R.string.widget_no_data)
                WidgetKind.MonthRing -> report?.let { context.getString(R.string.widget_month_values,
                    formatSeconds(it.workedSeconds), formatSeconds(it.plannedSeconds),
                    formatSeconds(it.overtimeSeconds), formatSeconds(it.undertimeSeconds)) } ?: context.getString(R.string.widget_no_data)
                else -> context.getString(R.string.widget_day_balance, formatSeconds(snapshot.balanceSeconds, true))
            })
            if (kind == WidgetKind.MonthBalance || kind == WidgetKind.MonthRing) {
                view.setTextViewText(R.id.widget_value, report?.let { formatSeconds(if (kind == WidgetKind.MonthBalance) it.balanceSeconds else it.workedSeconds, kind == WidgetKind.MonthBalance) } ?: "—")
            }
            if (snapshot.isWorking && kind in listOf(WidgetKind.Dashboard, WidgetKind.Compact, WidgetKind.Small) && !state.stale) {
                val elapsedSinceSnapshot = (System.currentTimeMillis() / 1000L - snapshot.sampledAtEpochSeconds).coerceAtLeast(0)
                val base = SystemClock.elapsedRealtime() - (snapshot.workedSeconds + elapsedSinceSnapshot) * 1000
                view.setChronometer(R.id.widget_clock, base, "%s", true)
                view.setViewVisibility(R.id.widget_clock, View.VISIBLE)
                view.setViewVisibility(R.id.widget_value, View.GONE)
            } else {
                view.setChronometer(R.id.widget_clock, SystemClock.elapsedRealtime(), "%s", false)
                view.setViewVisibility(R.id.widget_clock, View.GONE)
                view.setViewVisibility(R.id.widget_value, View.VISIBLE)
            }
            val bitmap = when(kind) {
                WidgetKind.DayTimeline -> painter.timeline(state.timeline)
                WidgetKind.MonthBalance -> painter.monthly(state.report)
                WidgetKind.MonthRing -> painter.progress(state.report?.workedSeconds ?: 0, state.report?.plannedSeconds ?: 0)
                WidgetKind.Tiny -> painter.tinyProgress(snapshot.workedSeconds, snapshot.requiredWorkSeconds)
                else -> painter.progress(snapshot.workedSeconds, snapshot.requiredWorkSeconds)
            }
            view.setImageViewBitmap(R.id.widget_chart, bitmap)
        }
        if (snapshot == null || !state.authenticated) {
            view.setChronometer(R.id.widget_clock, SystemClock.elapsedRealtime(), "%s", false)
            view.setViewVisibility(R.id.widget_clock, View.GONE)
            view.setViewVisibility(R.id.widget_value, View.VISIBLE)
            view.setImageViewBitmap(R.id.widget_chart, if (kind == WidgetKind.Tiny) painter.tinyProgress(0, 1) else painter.progress(0, 1))
        }
        val open = Intent(context, MainActivity::class.java)
        val openPending = PendingIntent.getActivity(context, widgetId, open, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        view.setOnClickPendingIntent(R.id.widget_root, openPending)
        val canClick = state.authenticated && snapshot != null && !state.stale
        if (kind == WidgetKind.Dashboard || kind == WidgetKind.Compact) {
            view.setTextViewText(R.id.widget_check_in, context.getString(R.string.ui_check_in))
            view.setTextViewText(R.id.widget_check_out, context.getString(R.string.ui_check_out))
        }
        if (kind == WidgetKind.Dashboard || kind == WidgetKind.Compact) for ((checkIn, id, key) in listOf(Triple(true, R.id.widget_check_in, 1), Triple(false, R.id.widget_check_out, 2))) {
            val intent = Intent(context, WidgetActionReceiver::class.java).apply {
                action = WidgetActionReceiver.ACTION
                putExtra(WidgetActionReceiver.WIDGET_ID, widgetId)
                putExtra(WidgetActionReceiver.CHECK_IN, checkIn)
            }
            val pending = PendingIntent.getBroadcast(context, widgetId * 10 + key, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            view.setOnClickPendingIntent(id, if (canClick && snapshot!!.isWorking != checkIn) pending else openPending)
            val enabled = canClick && snapshot!!.isWorking != checkIn
            view.setBoolean(id, "setEnabled", enabled)
            view.setTextColor(id, if (enabled) accent else secondary)
            if (Build.VERSION.SDK_INT >= 31) {
                val fill = context.getColor(if (dark) android.R.color.system_neutral2_800 else android.R.color.system_accent1_100)
                view.setColorStateList(id, "setBackgroundTintList", ColorStateList.valueOf(fill))
            }
        }
        return view
    }
}
