package com.whs31.taptime.app.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.ui.auth.AuthScreen
import com.whs31.taptime.app.ui.auth.AuthViewModel
import com.whs31.taptime.app.ui.auth.RegisterScreen
import com.whs31.taptime.app.ui.auth.RegisterViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory

@Composable
fun TapTimeApp(container: AppContainer) {
    val loggedIn by container.session.authenticated.collectAsState()
    var registration by rememberSaveable { mutableStateOf(false) }
    TapTimeTheme {
        Surface(modifier = Modifier.fillMaxSize()) {
            if (loggedIn) TapTimeNavHost(container)
            else if (registration) {
                val vm: RegisterViewModel = viewModel(factory = viewModelFactory { initializer { RegisterViewModel(container) } })
                RegisterScreen(vm, onBack = { registration = false })
            } else {
                val vm: AuthViewModel = viewModel(factory = viewModelFactory { initializer { AuthViewModel(container) } })
                AuthScreen(vm, onRegister = { registration = true })
            }
        }
    }
}
