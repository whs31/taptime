package com.whs31.taptime.app

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.whs31.taptime.app.rust.TapTimeSdk
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AuthenticationTest {
    @get:Rule val rule = createAndroidComposeRule<MainActivity>()

    @Test fun native_sdk_initializes_without_network() {
        TapTimeSdk("https://api.time.whs31.com").close()
    }

    @Test fun login_is_initial_screen_without_saved_session() {
        rule.onNodeWithText("TapTime").assertIsDisplayed()
        rule.onNodeWithText(rule.activity.getString(R.string.ui_sign_in)).assertIsDisplayed()
    }
}
