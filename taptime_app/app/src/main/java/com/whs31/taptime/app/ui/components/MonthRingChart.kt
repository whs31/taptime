package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun MonthRingChart(worked: Long, planned: Long) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Column { Text(stringResource(R.string.ui_004), style = MaterialTheme.typography.titleMedium)
            Text("${formatSeconds(worked)} / ${formatSeconds(planned)}") }
        DayProgressRing(worked, planned)
    }
}
