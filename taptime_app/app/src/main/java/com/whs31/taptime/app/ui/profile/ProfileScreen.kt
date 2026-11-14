package com.whs31.taptime.app.ui.profile

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import com.whs31.taptime.app.ui.components.ConfirmActionDialog

@Composable
fun ProfileScreen(vm: ProfileViewModel, back: () -> Unit) {
    val user by vm.user.collectAsState()
    val busy by vm.busy.collectAsState()
    val message by vm.message.collectAsState()
    var name by remember(user) { mutableStateOf(user?.name ?: "") }
    var email by remember(user) { mutableStateOf(user?.email ?: "") }
    var org by remember(user) { mutableStateOf(user?.organization ?: "") }
    var rfid by remember(user) { mutableStateOf(user?.rfidUid ?: "") }
    var deleteData by remember { mutableStateOf(false) }
    var deleteAccount by remember { mutableStateOf(false) }
    var password by remember { mutableStateOf("") }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        TextButton(onClick = back) { Text(stringResource(R.string.ui_052)) }
        Text(stringResource(R.string.ui_053), style = MaterialTheme.typography.headlineMedium)
        OutlinedTextField(name, { name = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_015)) })
        OutlinedTextField(email, { email = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_011)) })
        OutlinedTextField(org, { org = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_054)) })
        OutlinedTextField(rfid, { rfid = it }, Modifier.fillMaxWidth(), label = { Text("RFID UID") })
        message?.let { Text(it) }
        Button(enabled = !busy && user != null, onClick = { vm.save(name, email, org, rfid.ifBlank { null }) }) { Text(stringResource(R.string.ui_030)) }
        OutlinedButton(onClick = vm::logout, enabled = !busy) { Text(stringResource(R.string.ui_055)) }
        HorizontalDivider()
        Text(stringResource(R.string.ui_056), color = MaterialTheme.colorScheme.error)
        TextButton(onClick = { deleteData = true }, enabled = !busy) { Text(stringResource(R.string.ui_057)) }
        TextButton(onClick = { deleteAccount = true }, enabled = !busy) { Text(stringResource(R.string.ui_058)) }
    }
    if (deleteData) ConfirmActionDialog(stringResource(R.string.ui_delete_all), stringResource(R.string.ui_irreversible), { vm.deleteTimeData(); deleteData = false }, { deleteData = false })
    if (deleteAccount) AlertDialog(onDismissRequest = { deleteAccount = false }, title = { Text(stringResource(R.string.ui_059)) },
        text = { Column { Text(stringResource(R.string.ui_060)); OutlinedTextField(password, { password = it }, visualTransformation = PasswordVisualTransformation()) } },
        confirmButton = { TextButton(enabled = password.isNotEmpty(), onClick = { vm.deleteAccount(password); password = ""; deleteAccount = false }) { Text(stringResource(R.string.ui_023)) } },
        dismissButton = { TextButton(onClick = { deleteAccount = false }) { Text(stringResource(R.string.ui_009)) } })
}
