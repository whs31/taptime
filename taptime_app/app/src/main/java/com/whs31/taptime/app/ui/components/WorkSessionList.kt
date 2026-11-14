package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.rust.WorkSessionDto

@Composable
fun WorkSessionList(sessions: List<WorkSessionDto>) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(stringResource(R.string.ui_005), style = MaterialTheme.typography.titleMedium)
        if (sessions.isEmpty()) Text(stringResource(R.string.ui_006))
        sessions.forEach { session ->
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text("${formatClock(session.startSeconds)} — ${formatClock(session.endSeconds)}")
                Text(formatSeconds(session.workedSeconds))
            }
        }
    }
}
