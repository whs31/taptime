package com.whs31.taptime.app.ui.reports

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
import com.whs31.taptime.app.ui.components.*

@Composable
fun ReportsScreen(vm: ReportsViewModel) {
    val report by vm.report.collectAsState()
    val loading by vm.loading.collectAsState()
    val error by vm.error.collectAsState()
    val range by vm.range.collectAsState()
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.ui_037), style = MaterialTheme.typography.headlineMedium)
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            listOf("Месяц" to R.string.ui_month, "Прошлый месяц" to R.string.ui_previous_month, "Квартал" to R.string.ui_quarter, "Год" to R.string.ui_year).forEach { (label, resource) ->
                FilterChip(range == label, onClick = { vm.select(label) }, label = { Text(stringResource(resource)) })
            }
        }
        if (loading) LinearProgressIndicator(Modifier.fillMaxWidth())
        error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        report?.let { r ->
            Surface(shape = MaterialTheme.shapes.medium, modifier = Modifier.fillMaxWidth(),
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp)) { MonthRingChart(r.workedSeconds, r.plannedSeconds) }
            }
            HorizontalDivider()
Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Column { Text(stringResource(R.string.ui_038)); Text(formatSeconds(r.balanceSeconds, true), style = MaterialTheme.typography.titleLarge) }
                Column { Text(stringResource(R.string.ui_039)); Text(formatSeconds(r.overtimeSeconds)) }
                Column { Text(stringResource(R.string.ui_040)); Text(formatSeconds(r.undertimeSeconds)) }
            }

                }
            }
Surface(modifier = Modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(stringResource(R.string.ui_041), style = MaterialTheme.typography.titleMedium)
            MonthBalanceChart(r.dailyBalance)
            Text(stringResource(R.string.ui_working_days, r.distribution.workingDays.toLong(), r.distribution.workedDays.toLong()))
            Text(stringResource(R.string.ui_exception_days, r.exceptions.remoteDays.toLong(), r.exceptions.vacationDays.toLong(), r.exceptions.dayOffs.toLong()))

                }
            }
        }
        if (report == null && !loading) Text(stringResource(R.string.ui_035))
    }
}
