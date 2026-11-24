package com.whs31.taptime.app.ui.auth

import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp

@Composable
fun AuthScreen(vm: AuthViewModel, onRegister: () -> Unit) {
    var email by rememberSaveable { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    val busy by vm.busy.collectAsState()
    val error by vm.error.collectAsState()
    Column(Modifier.fillMaxSize().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
        Text("TapTime", style = MaterialTheme.typography.headlineLarge)
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(email, { email = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_011)) }, singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email))
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(password, { password = it }, Modifier.fillMaxWidth(), label = { Text(stringResource(R.string.ui_012)) }, singleLine = true,
            visualTransformation = PasswordVisualTransformation())
        if (error != null) Text(error ?: "", color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(8.dp))
        Spacer(Modifier.height(16.dp))
        Button(onClick = { vm.signIn(email, password) }, enabled = !busy && email.isNotBlank() && password.isNotBlank(), modifier = Modifier.fillMaxWidth()) {
            Text(stringResource(if (busy) R.string.ui_signing_in else R.string.ui_sign_in))
        }
        TextButton(onClick = onRegister) { Text(stringResource(R.string.ui_013)) }
    }
}
