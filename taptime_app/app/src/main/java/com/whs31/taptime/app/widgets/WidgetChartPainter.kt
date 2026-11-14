package com.whs31.taptime.app.widgets

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.util.TypedValue
import android.os.Build
import android.content.res.Configuration
import com.whs31.taptime.app.rust.*
import kotlin.math.max

class WidgetChartPainter(private val context: Context) {
    private val dark get() = context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES
    private fun primary() = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_accent1_200 else android.R.color.system_accent1_600) else resolve(android.R.attr.colorAccent, 0xff628ee1.toInt())
    private fun tertiary() = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_accent3_200 else android.R.color.system_accent3_600) else 0xffad7944.toInt()
    private fun resolve(attribute: Int, fallback: Int): Int {
        val value = TypedValue()
        return if (context.theme.resolveAttribute(attribute, value, true)) {
            if (value.resourceId != 0) runCatching { context.getColor(value.resourceId) }.getOrDefault(fallback) else value.data
        } else fallback
    }
    private val gray get() = if (Build.VERSION.SDK_INT >= 31) context.getColor(if (dark) android.R.color.system_neutral2_500 else android.R.color.system_neutral2_400) else 0xff9a9a9a.toInt()
    fun progress(worked: Long, target: Long, width: Int = 320, height: Int = 140): Bitmap {
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = minOf(width, height) * .09f; strokeCap = Paint.Cap.ROUND }
        val side = minOf(width, height) * .82f
        val left = (width - side) / 2
        val top = (height - side) / 2
        val box = android.graphics.RectF(left, top, left + side, top + side)
        paint.color = gray; paint.alpha = 90; canvas.drawArc(box, 0f, 360f, false, paint)
        paint.color = primary(); paint.alpha = 255
        val ratio = if (target > 0) (worked.toFloat() / target).coerceIn(0f, 1f) else 0f
        canvas.drawArc(box, -90f, ratio * 360f, false, paint)
        return bitmap
    }
    fun tinyProgress(worked: Long, target: Long, size: Int = 192): Bitmap {
        val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val centerX = size * .5f
        val centerY = size * .43f
        val radius = size * .32f
        val stroke = size * .09f
        val ring = android.graphics.RectF(centerX - radius, centerY - radius,
            centerX + radius, centerY + radius)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            style = Paint.Style.STROKE
            strokeWidth = stroke
            strokeCap = Paint.Cap.ROUND
        }
        paint.color = gray
        paint.alpha = 115
        canvas.drawArc(ring, 135f, 270f, false, paint)
        val progress = if (target > 0) (worked.toFloat() / target).coerceIn(0f, 1f) else 0f
        paint.color = primary()
        paint.alpha = 255
        if (progress > 0f) canvas.drawArc(ring, 135f, 270f * progress, false, paint)

        val iconY = centerY - size * .005f
        val iconRadius = size * .105f
        paint.color = if (Build.VERSION.SDK_INT >= 31) {
            context.getColor(if (dark) android.R.color.system_neutral1_50 else android.R.color.system_neutral1_900)
        } else if (dark) 0xfff0eded.toInt() else 0xff303030.toInt()
        paint.strokeWidth = size * .022f
        canvas.drawCircle(centerX, iconY, iconRadius, paint)
        canvas.drawLine(centerX, iconY, centerX, iconY - iconRadius * .53f, paint)
        canvas.drawLine(centerX, iconY, centerX + iconRadius * .45f, iconY + iconRadius * .2f, paint)
        return bitmap
    }
    fun timeline(timeline: TimelineDto?, width: Int = 500, height: Int = 96): Bitmap {
        val result = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(result)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = gray; alpha = 60 }
        val bounds = android.graphics.RectF(8f, height * .3f, width - 8f, height * .7f)
        canvas.drawRoundRect(bounds, 12f, 12f, paint)
        if (timeline == null) return result
        val span = (timeline.axisEndSeconds - timeline.axisStartSeconds).coerceAtLeast(1)
        for (part in timeline.segments) {
            val start = ((part.startSeconds - timeline.axisStartSeconds).toFloat() / span).coerceIn(0f, 1f)
            val end = ((part.endSeconds - timeline.axisStartSeconds).toFloat() / span).coerceIn(0f, 1f)
            if (start == end) continue
            paint.alpha = 255
            paint.color = when (part.kind) {
                TimelineSegmentKind.WORK -> primary()
                TimelineSegmentKind.BREAK -> tertiary()
                TimelineSegmentKind.OVERTIME -> tertiary()
            }
            canvas.drawRoundRect(android.graphics.RectF(8 + bounds.width() * start, bounds.top, 8 + bounds.width() * end, bounds.bottom), 5f, 5f, paint)
        }
        if (timeline.overtimeSeconds > 0) {
            paint.color = tertiary()
            paint.alpha = 255
            paint.textSize = 16f
            canvas.drawText("+" + com.whs31.taptime.app.ui.components.formatSeconds(timeline.overtimeSeconds),
                8f, height - 4f, paint)
        }
        timeline.nowSeconds?.let {
            val position = ((it - timeline.axisStartSeconds).toFloat() / span).coerceIn(0f, 1f)
            paint.color = gray; paint.alpha = 255; paint.strokeWidth = 3f
            val x = 8 + bounds.width() * position
            canvas.drawLine(x, 0f, x, height.toFloat(), paint)
        }
        return result
    }
    fun monthly(report: ReportDto?, width: Int = 500, height: Int = 120): Bitmap {
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val points = report?.dailyBalance ?: emptyList()
        if (points.isEmpty()) return bitmap
        val canvas = Canvas(bitmap)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = primary(); strokeWidth = 5f; style = Paint.Style.STROKE }
        val min = minOf(0L, points.minOf { it.cumulativeBalanceSeconds }).toFloat()
        val max = maxOf(0L, points.maxOf { it.cumulativeBalanceSeconds }).toFloat()
        val range = max(1f, max - min)
        val zeroY = 8f + (height - 16f) * max / range
        paint.color = gray
        paint.alpha = 90
        paint.strokeWidth = 1f
        canvas.drawLine(8f, zeroY, width - 8f, zeroY, paint)
        paint.color = primary()
        paint.alpha = 255
        paint.strokeWidth = 4f
        val path = android.graphics.Path()
        points.forEachIndexed { index, point ->
            val x = 8f + (width - 16f) * if (points.size == 1) .5f else index.toFloat() / (points.size - 1)
            val y = 8f + (height - 16f) * (max - point.cumulativeBalanceSeconds.toFloat()) / range
            if (index == 0) path.moveTo(x, y) else path.lineTo(x, y)
        }
        canvas.drawPath(path, paint)
        if (points.size == 1) {
            paint.style = Paint.Style.FILL
            val y = 8f + (height - 16f) * (max - points.first().cumulativeBalanceSeconds.toFloat()) / range
            canvas.drawCircle(width / 2f, y, 4f, paint)
        }
        return bitmap
    }
}
