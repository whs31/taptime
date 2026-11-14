package com.whs31.taptime.app.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.foundation.layout.Box
import androidx.compose.ui.unit.dp

@Composable
fun DayProgressRing(worked: Long, target: Long) {
    val progress = if (target > 0) (worked.toFloat() / target).coerceIn(0f, 1f) else 0f
    val primary = MaterialTheme.colorScheme.primary
    val muted = MaterialTheme.colorScheme.surfaceVariant
    Box(Modifier.size(104.dp), contentAlignment = Alignment.Center) {
        Canvas(Modifier.size(96.dp)) {
            drawArc(muted, -90f, 360f, false, style = Stroke(10.dp.toPx(), cap = StrokeCap.Round))
            drawArc(primary, -90f, progress * 360f, false, style = Stroke(10.dp.toPx(), cap = StrokeCap.Round))
        }
        Text("${(progress * 100).toInt()}%", style = MaterialTheme.typography.titleMedium)
    }
}
