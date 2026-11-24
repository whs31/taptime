package com.whs31.taptime.app.ui.auth

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp

@Composable
fun RegisterScreen(vm: RegisterViewModel, onBack: () -> Unit) {
    var name by rememberSaveable { mutableStateOf("") }
    var email by rememberSaveable { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    val busy by vm.busy.collectAsState()
    val error by vm.error.collectAsState()
    Column(Modifier.fillMaxSize().padding(24.dp), verticalArrangement = Arrangement.Center) {
        Text(stringResource(R.string.ui_014), style = MaterialTheme.typography.headlineMedium)
        OutlinedTextField(name, { name = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_015)) })
        OutlinedTextField(email, { email = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_011)) })
        OutlinedTextField(password, { password = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_012)) }, visualTransformation = PasswordVisualTransformation())
        if (error != null) Text(error ?: "", color = MaterialTheme.colorScheme.error)
        Button(onClick = { vm.register(name, email, password) }, enabled = !busy && name.isNotBlank() && email.isNotBlank() && password.length >= 8) { Text(stringResource(R.string.ui_013)) }
        TextButton(onClick = onBack) { Text(stringResource(R.string.ui_016)) }
    }
}
