package com.whs31.taptime.app

import androidx.compose.ui.test.assertIsDisplayed
import com.whs31.taptime.app.rust.ReportDto
import com.whs31.taptime.app.rust.ExceptionDto
import com.whs31.taptime.app.rust.WorkDistributionDto
import org.junit.Assert.assertTrue
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NavigationTest {
    @get:Rule val rule = createAndroidComposeRule<MainActivity>()

    @Test fun unauthenticated_navigation_never_exposes_private_screens() {
        rule.onNodeWithText(rule.activity.getString(R.string.ui_sign_in)).assertIsDisplayed()
        rule.onNodeWithText(rule.activity.getString(R.string.ui_000)).assertDoesNotExist()
    }
    @Test fun report_unsigned_counters_format_in_both_locales() {
        val report = ReportDto("2026-01-01", "2026-01-31", 0L, 0L, 0L, 0L, 0L,
            emptyList(), emptyList(), ExceptionDto(3u, 4u, 5u, 0u, 0u, 0u),
            WorkDistributionDto(21u, 10u, 19u, 0u, 0u))
        listOf(java.util.Locale.ENGLISH, java.util.Locale.forLanguageTag("ru")).forEach { locale ->
            val configuration = android.content.res.Configuration(rule.activity.resources.configuration)
            configuration.setLocale(locale)
            val context = rule.activity.createConfigurationContext(configuration)
            val working = context.getString(R.string.ui_working_days,
                report.distribution.workingDays.toLong(), report.distribution.workedDays.toLong())
            val exceptions = context.getString(R.string.ui_exception_days,
                report.exceptions.remoteDays.toLong(), report.exceptions.vacationDays.toLong(),
                report.exceptions.dayOffs.toLong())
            assertTrue(working.contains("21") && working.contains("19"))
            assertTrue(exceptions.contains("3") && exceptions.contains("5") && exceptions.contains("4"))
        }
    }
}
