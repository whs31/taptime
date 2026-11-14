package com.whs31.taptime.app.ui

import androidx.compose.foundation.layout.*
import androidx.compose.ui.res.stringResource
import com.whs31.taptime.app.R
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.whs31.taptime.app.AppContainer
import com.whs31.taptime.app.ui.today.*
import com.whs31.taptime.app.ui.history.*
import com.whs31.taptime.app.ui.reports.*
import com.whs31.taptime.app.ui.settings.*
import com.whs31.taptime.app.ui.profile.*

@Composable
fun TapTimeNavHost(container: AppContainer) {
    val nav = rememberNavController()
    val sections = listOf("today" to R.string.ui_000, "history" to R.string.ui_032, "reports" to R.string.ui_037, "settings" to R.string.ui_042)
    val backStackEntry by nav.currentBackStackEntryAsState()
    val selected = backStackEntry?.destination?.route ?: "today"
    Scaffold(bottomBar = {
        NavigationBar {
            sections.forEach { (route, label) ->
                NavigationBarItem(selected = selected == route, onClick = {
                    nav.navigate(route) { launchSingleTop = true; popUpTo("today") { saveState = true }; restoreState = true }
                }, icon = { Text(when (route) { "today" -> "◷"; "history" -> "▦"; "reports" -> "▥"; else -> "⚙" }) }, label = { Text(stringResource(label)) })
            }
        }
    }) { padding ->
        NavHost(navController = nav, startDestination = "today", modifier = Modifier.padding(padding)) {
            composable("today") {
                val vm: TodayViewModel = viewModel(factory = viewModelFactory { initializer { TodayViewModel(container) } })
                TodayScreen(vm)
            }
            composable("history") {
                val vm: HistoryViewModel = viewModel(factory = viewModelFactory { initializer { HistoryViewModel(container) } })
                HistoryScreen(vm)
            }
            composable("reports") {
                val vm: ReportsViewModel = viewModel(factory = viewModelFactory { initializer { ReportsViewModel(container) } })
                ReportsScreen(vm)
            }
            composable("settings") {
                val vm: SettingsViewModel = viewModel(factory = viewModelFactory { initializer { SettingsViewModel(container) } })
                SettingsScreen(vm, openProfile = { nav.navigate("profile") })
            }
            composable("profile") {
                val vm: ProfileViewModel = viewModel(factory = viewModelFactory { initializer { ProfileViewModel(container) } })
                ProfileScreen(vm, back = { nav.popBackStack() })
            }
        }
    }
}
