package com.whs31.taptime.app.ui.history

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.horizontalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.*
import com.whs31.taptime.app.ui.components.*
import java.time.LocalDate

@Composable
fun HistoryScreen(vm: HistoryViewModel) {
    val date by vm.date.collectAsState()
    val day by vm.day.collectAsState()
    val busy by vm.busy.collectAsState()
    val error by vm.error.collectAsState()
    var add by remember { mutableStateOf(false) }
    var delete by remember { mutableStateOf<String?>(null) }
    var override by remember { mutableStateOf(false) }
    var hours by remember { mutableStateOf("8") }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.ui_032), style = MaterialTheme.typography.headlineMedium)
        Surface(shape = MaterialTheme.shapes.medium, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 4.dp), verticalAlignment = androidx.compose.ui.Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            TextButton(onClick = { vm.load(LocalDate.parse(date).minusDays(1).toString()) }) { Text("‹") }
            Text(date, style = MaterialTheme.typography.titleLarge)
            TextButton(onClick = { vm.load(LocalDate.parse(date).plusDays(1).toString()) }) { Text("›") }
        }
        }
        if (busy) LinearProgressIndicator(Modifier.fillMaxWidth())
        error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        day?.let { entry ->
            Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(stringResource(R.string.ui_day_targets, formatSeconds(entry.requiredWorkSeconds), formatSeconds(entry.lunchSeconds)))
            Text(stringResource(R.string.ui_022), style = MaterialTheme.typography.titleMedium)
            if (entry.events.isEmpty()) Text(stringResource(R.string.ui_033))
            entry.events.forEach { event ->
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("${if (event.kind == EventKind.CHECK_IN) "Check In" else "Check Out"} · ${formatClock(event.localSeconds)}")
                    if (event.id.isNotEmpty()) TextButton(enabled = !busy, onClick = { delete = event.id }) { Text(stringResource(R.string.ui_023)) }
                }
            }
            Button(enabled = !busy, onClick = { add = true }) { Text(stringResource(R.string.ui_024)) }
                }
            }
            Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf(DayFlag.WEEKEND to R.string.ui_weekend, DayFlag.REMOTE to R.string.ui_remote, DayFlag.DAY_OFF to R.string.ui_day_off, DayFlag.VACATION to R.string.ui_vacation).forEach { (flag, label) ->
                    FilterChip(entry.flags.contains(flag), enabled = !busy, onClick = { vm.toggle(flag) }, label = { Text(stringResource(label)) })
                }
            }
            TextButton(onClick = { override = true }) { Text(stringResource(R.string.ui_034)) }
                }
            }
        }
        if (day == null && !busy) Text(stringResource(R.string.ui_035))
    }
    if (add) EventEditorSheet(date, onDismiss = { add = false }, onSave = { _, s, kind -> add = false; vm.add(s, kind) })
    delete?.let { id -> ConfirmActionDialog(stringResource(R.string.ui_delete_event), stringResource(R.string.ui_event_restore),
        confirm = { vm.delete(id); delete = null }, dismiss = { delete = null }) }
    if (override) AlertDialog(onDismissRequest = { override = false }, title = { Text(stringResource(R.string.ui_036)) },
        text = { OutlinedTextField(hours, { hours = it }, label = { Text(stringResource(R.string.ui_029)) }) },
        confirmButton = { TextButton(onClick = { hours.toLongOrNull()?.takeIf { it in 0..24 }?.let { vm.overrideHours(it * 3600) }; override = false }) { Text(stringResource(R.string.ui_030)) } },
        dismissButton = { TextButton(onClick = { vm.overrideHours(null); override = false }) { Text(stringResource(R.string.ui_031)) } })
}
