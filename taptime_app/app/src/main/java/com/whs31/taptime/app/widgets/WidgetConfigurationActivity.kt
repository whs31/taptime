package com.whs31.taptime.app.widgets

import android.app.Activity
import android.appwidget.AppWidgetManager
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.TapTimeApplication
import com.whs31.taptime.app.domain.TimelineMode
import com.whs31.taptime.app.ui.TapTimeTheme
import kotlinx.coroutines.launch

class WidgetConfigurationActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val id = intent?.getIntExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
            ?: AppWidgetManager.INVALID_APPWIDGET_ID
        if (id == AppWidgetManager.INVALID_APPWIDGET_ID) { finish(); return }
        setResult(Activity.RESULT_CANCELED)
        val app = (application as TapTimeApplication).container
        val prefs = WidgetPreferences(this)
        setContent {
            val scope = rememberCoroutineScope()
            var mode by rememberSaveable { mutableStateOf(prefs.mode(id, app.preferences.timelineMode)) }
            TapTimeTheme {
                Column(Modifier.fillMaxSize().padding(24.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(stringResource(R.string.widget_config), style = MaterialTheme.typography.headlineMedium)
                    Text(stringResource(R.string.widget_timeline_mode))
                    Row {
                        FilterChip(mode == TimelineMode.WORK_DAY, onClick = { mode = TimelineMode.WORK_DAY }, label = { Text(stringResource(R.string.widget_work_day)) })
                        Spacer(Modifier.width(12.dp))
                        FilterChip(mode == TimelineMode.CALENDAR_DAY, onClick = { mode = TimelineMode.CALENDAR_DAY }, label = { Text(stringResource(R.string.widget_calendar_day)) })
                    }
                    Button(onClick = {
                        prefs.setMode(id, mode)
                        scope.launch {
                            app.widgets.updateAll()
                            setResult(Activity.RESULT_OK, Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id))
                            finish()
                        }
                    }) { Text(stringResource(R.string.widget_save)) }
                    TextButton(onClick = { finish() }) { Text(stringResource(R.string.widget_cancel)) }
                }
            }
        }
    }
}
