package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.BorderStroke
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.DayMetricsDto
import kotlin.math.abs

fun formatSeconds(seconds: Long, signed: Boolean = false): String {
    val absolute = abs(seconds)
    val sign = if (signed) if (seconds < 0) "−" else "+" else ""
    return "%s%02d:%02d".format(sign, absolute / 3600, (absolute % 3600) / 60)
}
fun formatClock(seconds: Long?): String = seconds?.let { "%02d:%02d".format(it / 3600, (it % 3600) / 60) } ?: "—"

@Composable
fun WorkStatusCard(metrics: DayMetricsDto, monthBalance: Long?, busy: Boolean, onCheckIn: () -> Unit, onCheckOut: () -> Unit) {
    Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column {
                Text(stringResource(R.string.ui_000), style = MaterialTheme.typography.titleMedium)
                Text(formatSeconds(metrics.workedSeconds), style = MaterialTheme.typography.displayMedium, color = MaterialTheme.colorScheme.primary)
                Text(stringResource(R.string.ui_target, formatSeconds(metrics.requiredWorkSeconds)), color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            DayProgressRing(metrics.workedSeconds, metrics.requiredWorkSeconds)
        }
        Text(stringResource(R.string.ui_presence, metrics.presenceSeconds?.let { formatSeconds(it) } ?: "—"))
        HorizontalDivider()
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column { Text(stringResource(R.string.ui_001), style = MaterialTheme.typography.labelMedium); Text(formatSeconds(metrics.balanceSeconds, true)) }
            Column { Text(stringResource(R.string.ui_002), style = MaterialTheme.typography.labelMedium); Text(monthBalance?.let { formatSeconds(it, true) } ?: "—") }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(stringResource(R.string.ui_start, formatClock(metrics.firstCheckInSeconds)))
            Text(if (metrics.isWorking) stringResource(R.string.ui_working) else stringResource(R.string.ui_end, formatClock(metrics.lastCheckOutSeconds)))
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onCheckIn, enabled = !busy && !metrics.isWorking, modifier = Modifier.weight(1f)) { Text("Check In") }
            Button(onClick = onCheckOut, enabled = !busy && metrics.isWorking, modifier = Modifier.weight(1f)) { Text("Check Out") }
        }
    }
}
}
