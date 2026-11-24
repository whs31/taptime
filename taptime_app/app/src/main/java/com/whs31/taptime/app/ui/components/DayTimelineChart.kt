package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.*

@Composable
fun DayTimelineChart(timeline: TimelineDto) {
    val range = (timeline.axisEndSeconds - timeline.axisStartSeconds).coerceAtLeast(1).toFloat()
    val work = MaterialTheme.colorScheme.primary
    val breaks = MaterialTheme.colorScheme.tertiary
    val overtime = MaterialTheme.colorScheme.error
    Column {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(formatClock(timeline.axisStartSeconds), style = MaterialTheme.typography.labelSmall)
            Text(formatClock(timeline.axisEndSeconds), style = MaterialTheme.typography.labelSmall)
        }
        Canvas(Modifier.fillMaxWidth().height(24.dp)) {
            drawRect(Color.Gray.copy(alpha = .2f))
            timeline.segments.forEach { segment ->
                val from = ((segment.startSeconds - timeline.axisStartSeconds) / range).coerceIn(0f, 1f)
                val to = ((segment.endSeconds - timeline.axisStartSeconds) / range).coerceIn(0f, 1f)
                if (to > from) drawRect(when(segment.kind) {
                    TimelineSegmentKind.WORK -> work
                    TimelineSegmentKind.BREAK -> breaks
                    TimelineSegmentKind.OVERTIME -> overtime
                }, topLeft = Offset(from * size.width, 0f), size = androidx.compose.ui.geometry.Size((to - from) * size.width, size.height))
            }
            timeline.nowSeconds?.let {
                val x = ((it - timeline.axisStartSeconds) / range).coerceIn(0f, 1f) * size.width
                drawLine(Color.White, Offset(x, 0f), Offset(x, size.height), 2.dp.toPx())
            }
        }
        if (timeline.overtimeSeconds > 0) Text(stringResource(R.string.ui_timeline_overtime, formatSeconds(timeline.overtimeSeconds)))
    }
}
