package com.whs31.taptime.app

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.ui.TapTimeTheme
import com.whs31.taptime.app.ui.components.DayProgressRing
import org.junit.Rule
import org.junit.Test

class AdaptiveLayoutTest {
    @get:Rule val compose = createComposeRule()
    @Test fun ring_remains_visible_at_narrow_foldable_width() {
        compose.setContent { TapTimeTheme { Box(Modifier.width(240.dp)) { DayProgressRing(3600, 7200) } } }
        compose.onNodeWithText("50%").assertIsDisplayed()
    }
}
