package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun EventEditorSheet(date: String, onDismiss: () -> Unit, onSave: (String, Long, Boolean) -> Unit) {
    var time by remember { mutableStateOf("09:00") }
    var checkIn by remember { mutableStateOf(true) }
    val seconds = Regex("""^([01]\d|2[0-3]):([0-5]\d)$""").matchEntire(time)?.let { it.groupValues[1].toLong() * 3600 + it.groupValues[2].toLong() * 60 }
    AlertDialog(onDismissRequest = onDismiss, title = { Text(stringResource(R.string.ui_add_date, date)) },
        text = { Column {
            Row { FilterChip(checkIn, onClick = { checkIn = true }, label = { Text("Check In") }); Spacer(Modifier.width(8.dp)); FilterChip(!checkIn, onClick = { checkIn = false }, label = { Text("Check Out") }) }
            OutlinedTextField(time, { time = it }, label = { Text(stringResource(R.string.ui_007)) }, singleLine = true)
        } },
        confirmButton = { TextButton(onClick = { seconds?.let { onSave(date, it, checkIn) } }, enabled = seconds != null) { Text(stringResource(R.string.ui_008)) } },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.ui_009)) } })
}
