package com.whs31.taptime.app.ui.components

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.material3.*
import androidx.compose.runtime.Composable

@Composable
fun ConfirmActionDialog(title: String, explanation: String, confirm: () -> Unit, dismiss: () -> Unit) {
    AlertDialog(onDismissRequest = dismiss, title = { Text(title) }, text = { Text(explanation) },
        confirmButton = { TextButton(onClick = confirm) { Text(stringResource(R.string.ui_010)) } },
        dismissButton = { TextButton(onClick = dismiss) { Text(stringResource(R.string.ui_009)) } })
}
