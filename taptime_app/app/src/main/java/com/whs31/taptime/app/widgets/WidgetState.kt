package com.whs31.taptime.app.widgets

import com.whs31.taptime.app.data.DaySnapshotEntity
import com.whs31.taptime.app.rust.ReportDto
import com.whs31.taptime.app.rust.TimelineDto

class WidgetState(val authenticated: Boolean, val snapshot: DaySnapshotEntity?,
                  val timeline: TimelineDto?, val report: ReportDto?, val stale: Boolean = true, val remainingSeconds: Long? = null)
