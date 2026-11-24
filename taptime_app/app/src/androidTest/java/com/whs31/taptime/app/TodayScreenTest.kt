package com.whs31.taptime.app

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.assertIsDisplayed
import com.whs31.taptime.app.rust.DayMetricsDto
import com.whs31.taptime.app.ui.TapTimeTheme
import com.whs31.taptime.app.ui.components.WorkStatusCard
import org.junit.Rule
import org.junit.Test

class TodayScreenTest {
    @get:Rule val compose = createComposeRule()
    @Test fun day_status_shows_actions_without_using_network() {
        compose.setContent {
            TapTimeTheme {
                WorkStatusCard(DayMetricsDto("2026-11-14", false, 14400, 28800, 14400,
                    -14400, 14400, 32400, 46800, 61200, emptyList()), 0, false, {}, {})
            }
        }
        compose.onNodeWithText("Check In").assertIsDisplayed()
        compose.onNodeWithText("Check Out").assertIsDisplayed()
    }
}
