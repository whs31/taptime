package com.whs31.taptime.app.ui.today

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.*
import com.whs31.taptime.app.domain.TimelineMode as UiTimelineMode
import com.whs31.taptime.app.ui.components.*

@Composable
fun TodayScreen(vm: TodayViewModel) {
    val current by vm.today.collectAsState()
    val busy by vm.busy.collectAsState()
    val message by vm.message.collectAsState()
    val timeline by vm.timeline.collectAsState()
    val mode by vm.mode.collectAsState()
    var edit by remember { mutableStateOf(false) }
    var remove by remember { mutableStateOf<String?>(null) }
    var override by remember { mutableStateOf(false) }
    var overrideHours by remember { mutableStateOf("8") }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text("TapTime", style = MaterialTheme.typography.headlineMedium)
            TextButton(onClick = vm::refresh, enabled = !busy) { Text(stringResource(R.string.ui_017)) }
        }
        if (current.loading) LinearProgressIndicator(Modifier.fillMaxWidth())
        if (current.stale) Text(stringResource(R.string.ui_018), color = MaterialTheme.colorScheme.error)
        current.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        current.metrics?.let { metrics ->
            WorkStatusCard(metrics, current.monthlyBalanceSeconds, busy,
                onCheckIn = { vm.action(true) }, onCheckOut = { vm.action(false) })
            Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(stringResource(R.string.ui_019), style = MaterialTheme.typography.titleMedium)
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        FilterChip(mode == UiTimelineMode.WORK_DAY, onClick = { vm.selectMode(UiTimelineMode.WORK_DAY) }, label = { Text(stringResource(R.string.ui_020)) })
                        FilterChip(mode == UiTimelineMode.CALENDAR_DAY, onClick = { vm.selectMode(UiTimelineMode.CALENDAR_DAY) }, label = { Text(stringResource(R.string.ui_021)) })
                    }
                    timeline?.let { DayTimelineChart(it) }
                    HorizontalDivider()
                    WorkSessionList(metrics.sessions)
                }
            }
            Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(stringResource(R.string.ui_022), style = MaterialTheme.typography.titleMedium)
                    current.day?.events?.forEach { event ->
                        Row(Modifier.fillMaxWidth(), verticalAlignment = androidx.compose.ui.Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
                            Text("${stringResource(if (event.kind == EventKind.CHECK_IN) R.string.ui_check_in else R.string.ui_check_out)} · ${formatClock(event.localSeconds)}")
                            if (event.id.isNotBlank()) TextButton(onClick = { remove = event.id }, enabled = !busy) { Text(stringResource(R.string.ui_023)) }
                        }
                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                    }
                    OutlinedButton(onClick = { edit = true }, enabled = !busy, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.ui_024)) }
                }
            }
            Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(stringResource(R.string.ui_025), style = MaterialTheme.typography.titleMedium)
                    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        listOf(DayFlag.WEEKEND to R.string.ui_weekend, DayFlag.REMOTE to R.string.ui_remote, DayFlag.DAY_OFF to R.string.ui_day_off, DayFlag.VACATION to R.string.ui_vacation).forEach { (flag, label) ->
                            FilterChip(current.day?.flags?.contains(flag) == true, onClick = { vm.toggle(metrics.date, flag) }, enabled = !busy, label = { Text(stringResource(label)) })
                        }
                    }
                    TextButton(onClick = { override = true }, enabled = !busy) { Text(stringResource(R.string.ui_026)) }
                }
            }
        }
        if (current.metrics == null && !current.loading) Text(stringResource(R.string.ui_027))
    }
    val date = current.metrics?.date
    if (edit && date != null) EventEditorSheet(date, onDismiss = { edit = false }, onSave = { d, s, inEvent -> edit = false; vm.add(d, s, inEvent) })
    remove?.let { id -> ConfirmActionDialog(stringResource(R.string.ui_delete_event), stringResource(R.string.ui_event_change),
        confirm = { vm.delete(id); remove = null }, dismiss = { remove = null }) }
    if (override && date != null) AlertDialog(onDismissRequest = { override = false },
        title = { Text(stringResource(R.string.ui_028)) }, text = {
            OutlinedTextField(overrideHours, { overrideHours = it }, label = { Text(stringResource(R.string.ui_029)) })
        }, confirmButton = { TextButton(onClick = {
            overrideHours.toLongOrNull()?.takeIf { it in 0..24 }?.let { vm.setOverride(date, it * 3600) }
            override = false
        }) { Text(stringResource(R.string.ui_030)) } }, dismissButton = { TextButton(onClick = { vm.setOverride(date, null); override = false }) { Text(stringResource(R.string.ui_031)) } })
}
