package com.whs31.taptime.app.ui

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun TapTimeTheme(content: @Composable () -> Unit) {
    val context = LocalContext.current
    val dark = isSystemInDarkTheme()
    val dynamic = context.getSharedPreferences("taptime-preferences", 0).getBoolean("dynamic_color", false)
    val darkPalette = darkColorScheme(
        primary = Color(0xFFF3C344), onPrimary = Color(0xFF302500),
        primaryContainer = Color(0xFF443713), onPrimaryContainer = Color(0xFFFFE8A1),
        secondary = Color(0xFFD3C3A0), secondaryContainer = Color(0xFF34302A),
        background = Color(0xFF0F0E0D), onBackground = Color(0xFFF1EDE7),
        surface = Color(0xFF211E1B), onSurface = Color(0xFFF1EDE7),
        surfaceVariant = Color(0xFF2A2622), onSurfaceVariant = Color(0xFFB7AEA2),
        outline = Color(0xFF655C51), outlineVariant = Color(0xFF39332C),
        error = Color(0xFFFFB4AB)
    )
    val lightPalette = lightColorScheme(
        primary = Color(0xFF785900), onPrimary = Color.White,
        primaryContainer = Color(0xFFFFE8A1), onPrimaryContainer = Color(0xFF271C00),
        secondary = Color(0xFF6A5E47), background = Color(0xFFFFFBF5),
        surface = Color(0xFFFFFBF5), surfaceVariant = Color(0xFFF0E9DE),
        onSurface = Color(0xFF211E19), onSurfaceVariant = Color(0xFF665F54),
        outlineVariant = Color(0xFFE0D8CB)
    )
    val scheme = if (dynamic && Build.VERSION.SDK_INT >= 31) {
        if (dark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
    } else if (dark) darkPalette else lightPalette
    val typography = Typography(
        headlineMedium = TextStyle(fontSize = 25.sp, lineHeight = 32.sp, fontWeight = FontWeight.SemiBold),
        titleLarge = TextStyle(fontSize = 20.sp, lineHeight = 26.sp, fontWeight = FontWeight.Medium),
        titleMedium = TextStyle(fontSize = 15.sp, lineHeight = 22.sp, fontWeight = FontWeight.SemiBold),
        bodyLarge = TextStyle(fontSize = 15.sp, lineHeight = 22.sp),
        bodyMedium = TextStyle(fontSize = 14.sp, lineHeight = 20.sp),
        labelLarge = TextStyle(fontSize = 14.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium),
        displayMedium = TextStyle(fontSize = 40.sp, lineHeight = 48.sp, fontWeight = FontWeight.Medium)
    )
    MaterialTheme(colorScheme = scheme, typography = typography,
        shapes = Shapes(extraSmall = RoundedCornerShape(4.dp), small = RoundedCornerShape(6.dp),
            medium = RoundedCornerShape(10.dp), large = RoundedCornerShape(12.dp), extraLarge = RoundedCornerShape(16.dp)),
        content = content)
}
