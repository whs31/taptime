package com.whs31.taptime.app.ui.settings

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
import com.whs31.taptime.app.domain.TimelineMode

@Composable
fun SettingsScreen(vm: SettingsViewModel, openProfile: () -> Unit) {
    val user by vm.user.collectAsState()
    val busy by vm.busy.collectAsState()
    val message by vm.message.collectAsState()
    var zone by remember(user) { mutableStateOf(user?.timeZone ?: "") }
    var work by remember(user) { mutableStateOf(((user?.settings?.requiredWorkSeconds ?: 28800L) / 3600).toString()) }
    var lunch by remember(user) { mutableStateOf(((user?.settings?.lunchSeconds ?: 1800L) / 60).toString()) }
    var start by remember(user) { mutableStateOf(user?.settings?.startDate ?: "") }
    var weekends by remember(user) { mutableStateOf(user?.settings?.weekends ?: emptyList()) }
    var remote by remember(user) { mutableStateOf(user?.settings?.remoteDays ?: emptyList()) }
    var mode by remember { mutableStateOf(vm.preferences.timelineMode) }
    val days = Weekday.entries.mapIndexed { index, day -> day to
        java.time.DayOfWeek.of(index + 1).getDisplayName(java.time.format.TextStyle.SHORT, java.util.Locale.getDefault()) }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.ui_042), style = MaterialTheme.typography.headlineMedium)
        TextButton(onClick = openProfile) { Text(stringResource(R.string.ui_043)) }
Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        OutlinedTextField(zone, { zone = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_044)) })
        OutlinedTextField(work, { work = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_045)) })
        OutlinedTextField(lunch, { lunch = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_046)) })
        OutlinedTextField(start, { start = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_047)) })

                }
            }
Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.ui_048), style = MaterialTheme.typography.titleMedium)
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            days.forEach { (d, label) -> FilterChip(weekends.contains(d), onClick = { weekends = if (d in weekends) weekends - d else weekends + d }, label = { Text(label) }) }
        }
        Text(stringResource(R.string.ui_049), style = MaterialTheme.typography.titleMedium)
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            days.forEach { (d, label) -> FilterChip(remote.contains(d), onClick = { remote = if (d in remote) remote - d else remote + d }, label = { Text(label) }) }
        }

                }
            }
Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.ui_050), style = MaterialTheme.typography.titleMedium)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            FilterChip(mode == TimelineMode.WORK_DAY, onClick = { mode = TimelineMode.WORK_DAY; vm.preferences.timelineMode = mode }, label = { Text(stringResource(R.string.ui_020)) })
            FilterChip(mode == TimelineMode.CALENDAR_DAY, onClick = { mode = TimelineMode.CALENDAR_DAY; vm.preferences.timelineMode = mode }, label = { Text(stringResource(R.string.ui_021)) })
        }

                }
            }
        message?.let { Text(it) }
        Button(onClick = { vm.save(zone, work.toLongOrNull() ?: -1, lunch.toLongOrNull() ?: -1, weekends, remote, start.ifBlank { null }) }, enabled = !busy && user != null) { Text(stringResource(R.string.ui_051)) }
    }
}
