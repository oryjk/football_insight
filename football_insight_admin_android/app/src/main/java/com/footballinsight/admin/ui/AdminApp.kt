@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package com.footballinsight.admin.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.Logout
import androidx.compose.material.icons.filled.AdminPanelSettings
import androidx.compose.material.icons.filled.Badge
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Group
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.ManageAccounts
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Switch
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.footballinsight.admin.data.remote.AdminAuditLogDto
import com.footballinsight.admin.data.remote.AdminUserDto

@Composable
fun AdminApp(viewModel: AdminViewModel, onBiometricUnlock: () -> Unit) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(state.error, state.message) {
        val notice = state.error ?: state.message
        if (notice != null) {
            snackbar.showSnackbar(notice)
            viewModel.consumeNotice()
        }
    }

    Box(Modifier.fillMaxSize()) {
        when {
            state.requiresBiometricUnlock -> LocalUnlockPane(onBiometricUnlock)
            state.checkingSession -> LoadingPane()
            !state.authenticated -> LoginScreen(
                state.loading,
                state.serverUrl,
                viewModel::login,
                viewModel::switchServer,
            )
            state.selectedUser != null -> UserDetailScreen(
                state.selectedUser!!,
                state.loading,
                viewModel::closeUser,
                viewModel::changeStatus,
                viewModel::adjustMembership,
            )
            else -> AdminHome(state, viewModel, snackbar)
        }
        if (state.loading && state.authenticated) {
            Surface(Modifier.align(Alignment.TopCenter).padding(top = 72.dp), tonalElevation = 4.dp) {
                Row(Modifier.padding(horizontal = 14.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                    Text("正在同步", Modifier.padding(start = 8.dp), style = MaterialTheme.typography.labelMedium)
                }
            }
        }
        if (!state.authenticated) {
            SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).padding(16.dp))
        }
    }
}

@Composable
private fun LoginScreen(
    loading: Boolean,
    serverUrl: String,
    onLogin: (String, String) -> Unit,
    onSwitchServer: (String) -> Unit,
) {
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var serverDraft by remember(serverUrl) { mutableStateOf(serverUrl) }
    Column(
        Modifier.fillMaxSize().padding(horizontal = 28.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(Icons.Default.AdminPanelSettings, null, Modifier.size(44.dp), tint = MaterialTheme.colorScheme.primary)
        Spacer(Modifier.height(18.dp))
        Text("足球洞察管理", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Text("管理员登录", color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(28.dp))
        OutlinedTextField(
            username,
            { username = it },
            Modifier.fillMaxWidth(),
            label = { Text("用户名") },
            singleLine = true,
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            password,
            { password = it },
            Modifier.fillMaxWidth(),
            label = { Text("密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { onLogin(username, password) }),
        )
        Spacer(Modifier.height(18.dp))
        Button(
            onClick = { onLogin(username, password) },
            enabled = !loading && username.isNotBlank() && password.length >= 8,
            modifier = Modifier.fillMaxWidth().height(48.dp),
        ) { Text(if (loading) "登录中" else "登录") }
        OutlinedTextField(
            serverDraft,
            { serverDraft = it },
            Modifier.fillMaxWidth().padding(top = 16.dp),
            label = { Text("服务器地址") },
            singleLine = true,
        )
        OutlinedButton(
            onClick = { onSwitchServer(serverDraft) },
            enabled = !loading && serverDraft.isNotBlank(),
            modifier = Modifier.padding(top = 8.dp),
        ) { Text("验证服务器") }
    }
}

@Composable
private fun AdminHome(state: AdminUiState, viewModel: AdminViewModel, snackbar: SnackbarHostState) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text(destinationTitle(state.destination), fontWeight = FontWeight.SemiBold)
                        Text(state.displayName, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                },
                actions = {
                    IconButton(onClick = {
                        when (state.destination) {
                            AdminDestination.Users -> viewModel.loadUsers()
                            AdminDestination.Audit -> viewModel.loadAudit()
                            AdminDestination.Settings -> Unit
                        }
                    }) { Icon(Icons.Default.Refresh, "刷新") }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
            )
        },
        bottomBar = {
            NavigationBar {
                AdminDestination.entries.forEach { destination ->
                    NavigationBarItem(
                        selected = state.destination == destination,
                        onClick = { viewModel.navigate(destination) },
                        icon = { Icon(destinationIcon(destination), destinationTitle(destination)) },
                        label = { Text(destinationTitle(destination)) },
                    )
                }
            }
        },
        snackbarHost = { SnackbarHost(snackbar) },
    ) { padding ->
        when (state.destination) {
            AdminDestination.Users -> UsersScreen(state, viewModel, Modifier.padding(padding))
            AdminDestination.Audit -> AuditScreen(state.auditLogs, Modifier.padding(padding))
            AdminDestination.Settings -> SettingsScreen(state, viewModel, Modifier.padding(padding))
        }
    }
}

@Composable
private fun UsersScreen(state: AdminUiState, viewModel: AdminViewModel, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxSize()) {
        OutlinedTextField(
            state.query,
            viewModel::updateQuery,
            Modifier.fillMaxWidth().padding(horizontal = 16.dp),
            leadingIcon = { Icon(Icons.Default.Search, null) },
            trailingIcon = { IconButton(onClick = viewModel::loadUsers) { Icon(Icons.Default.Search, "搜索") } },
            placeholder = { Text("账号或昵称") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            keyboardActions = KeyboardActions(onSearch = { viewModel.loadUsers() }),
        )
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            FilterChip(state.statusFilter == null, { viewModel.setFilters(null, state.tierFilter) }, { Text("全部") })
            FilterChip(state.statusFilter == "active", { viewModel.setFilters("active", state.tierFilter) }, { Text("正常") })
            FilterChip(state.statusFilter == "disabled", { viewModel.setFilters("disabled", state.tierFilter) }, { Text("已禁用") })
        }
        Text("${state.totalUsers} 位用户", Modifier.padding(horizontal = 18.dp, vertical = 4.dp), style = MaterialTheme.typography.labelMedium)
        if (state.users.isEmpty() && !state.loading) {
            EmptyPane("没有符合条件的用户")
        } else {
            LazyColumn(Modifier.fillMaxSize()) {
                items(state.users, key = { it.id }) { user ->
                    ListItem(
                        headlineContent = { Text(user.displayName, fontWeight = FontWeight.Medium) },
                        supportingContent = { Text("${user.accountIdentifier}  ·  ${user.membershipTier}") },
                        leadingContent = {
                            Icon(
                                if (user.status == "active") Icons.Default.CheckCircle else Icons.Default.Block,
                                null,
                                tint = if (user.status == "active") MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error,
                            )
                        },
                        trailingContent = { Text(if (user.status == "active") "正常" else "禁用", style = MaterialTheme.typography.labelMedium) },
                        modifier = Modifier.fillMaxWidth(),
                    )
                    TextButton(onClick = { viewModel.openUser(user.id) }, modifier = Modifier.padding(start = 58.dp)) { Text("查看与管理") }
                    HorizontalDivider()
                }
            }
        }
    }
}

@Composable
private fun UserDetailScreen(
    user: AdminUserDto,
    loading: Boolean,
    onBack: () -> Unit,
    onStatus: (AdminUserDto, String) -> Unit,
    onMembership: (AdminUserDto, String, String, String?, String) -> Unit,
) {
    var statusDialog by remember { mutableStateOf(false) }
    var membershipDialog by remember { mutableStateOf(false) }
    Scaffold(topBar = {
        TopAppBar(
            title = { Text(user.displayName) },
            navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回") } },
        )
    }) { padding ->
        LazyColumn(Modifier.fillMaxSize().padding(padding)) {
            item {
                DetailSection("账号", listOf("登录名" to user.accountIdentifier, "用户 ID" to user.id, "状态" to if (user.status == "active") "正常" else "已禁用"))
                DetailSection("会员", listOf("等级" to user.membershipTier, "到期" to (user.membershipExpiresAt ?: "长期有效")))
                DetailSection("邀请关系", listOf("邀请码" to (user.inviteCode ?: "无"), "邀请人" to (user.invitedBy?.displayName ?: "无")))
                DetailSection("账号信息", listOf("微信绑定" to if (user.hasWechatBinding) "已绑定" else "未绑定", "创建时间" to compactTime(user.createdAt), "更新时间" to compactTime(user.updatedAt)))
                user.activity?.let { activity ->
                    DetailSection(
                        "活跃",
                        listOf(
                            "最近登录" to (activity.lastLoginAt?.let(::compactTime) ?: "无"),
                            "最近活跃" to (activity.lastActiveAt?.let(::compactTime) ?: "无"),
                            "最近页面" to (activity.lastActivePageKey ?: "无"),
                        ),
                    )
                }
                DetailSection("邀请下级", listOf("人数" to user.referrals.orEmpty().size.toString()))
                user.referrals.orEmpty().forEach { referral ->
                    DetailSection(referral.displayName, listOf("账号" to referral.accountIdentifier, "状态" to referral.status, "加入" to compactTime(referral.createdAt)))
                }
                DetailSection("订单", listOf("最近记录" to user.orders.orEmpty().size.toString()))
                user.orders.orEmpty().forEach { order ->
                    DetailSection(order.orderNo, listOf("产品" to order.productType, "金额" to "¥%.2f".format(order.amountCents / 100.0), "状态" to order.status, "创建" to compactTime(order.createdAt)))
                }
                DetailSection("订阅", listOf("记录数" to user.subscriptions.orEmpty().size.toString()))
                user.subscriptions.orEmpty().forEach { subscription ->
                    DetailSection(subscription.planCode, listOf("球队" to subscription.teamCode, "范围" to subscription.scope, "状态" to subscription.status, "到期" to (subscription.expiresAt?.let(::compactTime) ?: "长期")))
                }
                DetailSection("设备", listOf("设备数" to user.devices.orEmpty().size.toString()))
                user.devices.orEmpty().forEach { device ->
                    DetailSection(device.platform.uppercase(), listOf("令牌" to device.maskedDeviceToken, "更新" to compactTime(device.updatedAt)))
                }
                Row(Modifier.fillMaxWidth().padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Button(onClick = { membershipDialog = true }, enabled = !loading, modifier = Modifier.weight(1f)) {
                        Icon(Icons.Default.Badge, null)
                        Text("调整会员", Modifier.padding(start = 6.dp))
                    }
                    OutlinedButton(onClick = { statusDialog = true }, enabled = !loading, modifier = Modifier.weight(1f)) {
                        Icon(if (user.status == "active") Icons.Default.Block else Icons.Default.CheckCircle, null)
                        Text(if (user.status == "active") "禁用" else "恢复", Modifier.padding(start = 6.dp))
                    }
                }
            }
        }
    }
    if (statusDialog) ReasonDialog(
        title = if (user.status == "active") "禁用用户" else "恢复用户",
        confirmLabel = if (user.status == "active") "确认禁用" else "确认恢复",
        onDismiss = { statusDialog = false },
        onConfirm = { reason -> statusDialog = false; onStatus(user, reason) },
    )
    if (membershipDialog) MembershipDialog(user, { membershipDialog = false }) { tier, mode, expires, reason ->
        membershipDialog = false
        onMembership(user, tier, mode, expires, reason)
    }
}

@Composable
private fun DetailSection(title: String, values: List<Pair<String, String>>) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 12.dp)) {
        Text(title, style = MaterialTheme.typography.titleSmall, color = MaterialTheme.colorScheme.primary)
        values.forEach { (label, value) ->
            Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text(value, modifier = Modifier.padding(start = 20.dp), fontWeight = FontWeight.Medium)
            }
        }
    }
    HorizontalDivider()
}

@Composable
private fun ReasonDialog(title: String, confirmLabel: String, onDismiss: () -> Unit, onConfirm: (String) -> Unit) {
    var reason by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title) },
        text = { OutlinedTextField(reason, { reason = it }, label = { Text("原因") }, minLines = 2) },
        confirmButton = { TextButton(onClick = { onConfirm(reason) }, enabled = reason.isNotBlank()) { Text(confirmLabel) } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } },
    )
}

@Composable
private fun MembershipDialog(user: AdminUserDto, onDismiss: () -> Unit, onConfirm: (String, String, String?, String) -> Unit) {
    var tier by remember { mutableStateOf(user.membershipTier) }
    var mode by remember { mutableStateOf("preserve") }
    var expiresAt by remember { mutableStateOf("") }
    var reason by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("调整会员") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(tier, { tier = it.uppercase() }, label = { Text("等级 V1 - V9") }, singleLine = true)
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    FilterChip(mode == "preserve", { mode = "preserve" }, { Text("保留到期") })
                    FilterChip(mode == "never", { mode = "never" }, { Text("长期") })
                    FilterChip(mode == "specific", { mode = "specific" }, { Text("指定") })
                }
                if (mode == "specific") OutlinedTextField(expiresAt, { expiresAt = it }, label = { Text("到期时间（RFC3339）") })
                OutlinedTextField(reason, { reason = it }, label = { Text("调整原因") }, minLines = 2)
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onConfirm(tier, mode, expiresAt.ifBlank { null }, reason) },
                enabled = tier.matches(Regex("V[1-9]")) && reason.isNotBlank() && (mode != "specific" || expiresAt.isNotBlank()),
            ) { Text("确认调整") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } },
    )
}

@Composable
private fun AuditScreen(logs: List<AdminAuditLogDto>, modifier: Modifier = Modifier) {
    if (logs.isEmpty()) {
        Box(modifier.fillMaxSize()) { EmptyPane("暂无审计记录") }
        return
    }
    LazyColumn(modifier.fillMaxSize()) {
        items(logs, key = { it.id }) { log -> AuditRow(log) }
    }
}

@Composable
private fun AuditRow(log: AdminAuditLogDto) {
    ListItem(
        headlineContent = { Text(actionLabel(log.action), fontWeight = FontWeight.Medium) },
        supportingContent = { Text(listOfNotNull(log.adminUsername, log.reason).joinToString(" · ")) },
        leadingContent = { Icon(Icons.Default.History, null) },
        trailingContent = { Text(compactTime(log.createdAt), style = MaterialTheme.typography.labelSmall) },
    )
    HorizontalDivider()
}

@Composable
private fun SettingsScreen(state: AdminUiState, viewModel: AdminViewModel, modifier: Modifier = Modifier) {
    var server by remember(state.serverUrl) { mutableStateOf(state.serverUrl) }
    LazyColumn(modifier.fillMaxSize()) {
        item {
            DetailSection("管理员", listOf("姓名" to state.displayName, "用户名" to state.username))
            Column(Modifier.fillMaxWidth().padding(18.dp)) {
                Text("服务器", style = MaterialTheme.typography.titleSmall, color = MaterialTheme.colorScheme.primary)
                OutlinedTextField(server, { server = it }, Modifier.fillMaxWidth().padding(top = 10.dp), label = { Text("API 基址") }, singleLine = true)
                Button(onClick = { viewModel.switchServer(server) }, Modifier.padding(top = 10.dp), enabled = !state.loading) {
                    Icon(Icons.Default.Settings, null)
                    Text("验证并切换", Modifier.padding(start = 6.dp))
                }
                Text("模拟器本机服务：http://10.0.2.2:8080/", Modifier.padding(top = 8.dp), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.outline)
            }
            HorizontalDivider()
            ListItem(
                headlineContent = { Text("生物识别解锁") },
                supportingContent = { Text("再次打开 App 时验证本机身份") },
                leadingContent = { Icon(Icons.Default.Badge, null) },
                trailingContent = {
                    Switch(
                        checked = state.biometricEnabled,
                        onCheckedChange = viewModel::setBiometricEnabled,
                    )
                },
            )
            HorizontalDivider()
            OutlinedButton(onClick = viewModel::logout, Modifier.fillMaxWidth().padding(18.dp)) {
                Icon(Icons.AutoMirrored.Filled.Logout, null)
                Text("退出登录", Modifier.padding(start = 8.dp))
            }
        }
    }
}

@Composable
private fun LoadingPane() = Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { CircularProgressIndicator() }

@Composable
private fun LocalUnlockPane(onUnlock: () -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(28.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(Icons.Default.AdminPanelSettings, null, Modifier.size(48.dp), tint = MaterialTheme.colorScheme.primary)
        Text("足球洞察管理", Modifier.padding(top = 16.dp), style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
        Button(onClick = onUnlock, Modifier.padding(top = 24.dp)) { Text("验证并解锁") }
    }
}

@Composable
private fun EmptyPane(text: String) = Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { Text(text, color = MaterialTheme.colorScheme.outline) }

private fun compactTime(value: String): String = value.replace('T', ' ').take(16)
private fun actionLabel(value: String): String = when (value) {
    "user.disable" -> "禁用用户"
    "user.restore" -> "恢复用户"
    "user.membership.adjust" -> "调整会员"
    else -> value
}
private fun destinationTitle(value: AdminDestination): String = when (value) {
    AdminDestination.Users -> "用户"
    AdminDestination.Audit -> "审计"
    AdminDestination.Settings -> "设置"
}
private fun destinationIcon(value: AdminDestination) = when (value) {
    AdminDestination.Users -> Icons.Default.Group
    AdminDestination.Audit -> Icons.Default.ManageAccounts
    AdminDestination.Settings -> Icons.Default.Settings
}
