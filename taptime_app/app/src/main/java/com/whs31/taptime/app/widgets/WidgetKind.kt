package com.whs31.taptime.app.widgets

import com.whs31.taptime.app.R

@Suppress("EnumEntryName")
enum class WidgetKind(val layoutId: Int, val label: String) {
    Dashboard(R.layout.widget_dashboard, "TapTime"),
    DayTimeline(R.layout.widget_day_timeline, "День"),
    MonthBalance(R.layout.widget_month_balance, "Баланс месяца"),
    MonthRing(R.layout.widget_month_ring, "Прогресс месяца"),
    Compact(R.layout.widget_compact, "TapTime"),
    Small(R.layout.widget_small, "TapTime"),
    Tiny(R.layout.widget_tiny, "TapTime");
}
