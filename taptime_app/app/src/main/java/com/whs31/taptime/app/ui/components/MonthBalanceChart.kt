package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.DailyBalancePointDto

@Composable
fun MonthBalanceChart(points: List<DailyBalancePointDto>) {
    if (points.isEmpty()) { Text(stringResource(R.string.ui_003)); return }
    val lineColor = MaterialTheme.colorScheme.primary
    val zeroColor = MaterialTheme.colorScheme.outlineVariant
    Canvas(Modifier.fillMaxWidth().height(164.dp)) {
        val min = minOf(0L, points.minOf { it.cumulativeBalanceSeconds }).toFloat()
        val max = maxOf(0L, points.maxOf { it.cumulativeBalanceSeconds }).toFloat()
        val range = (max - min).coerceAtLeast(1f)
        fun y(value: Long) = size.height * (max - value.toFloat()) / range
        drawLine(zeroColor, Offset(0f, y(0)), Offset(size.width, y(0)), 1.dp.toPx())
        val path = Path()
        points.forEachIndexed { index, point ->
            val x = if(points.size > 1) size.width * index / (points.size - 1) else size.width / 2
            if(index == 0) path.moveTo(x, y(point.cumulativeBalanceSeconds)) else path.lineTo(x, y(point.cumulativeBalanceSeconds))
        }
        drawPath(path, lineColor, style = Stroke(3.dp.toPx()))
    }
}
