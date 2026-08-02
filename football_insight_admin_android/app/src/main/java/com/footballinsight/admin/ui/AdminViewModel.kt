package com.footballinsight.admin.ui

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.footballinsight.admin.data.remote.AdminAuditLogDto
import com.footballinsight.admin.data.remote.AdminLoginRequest
import com.footballinsight.admin.data.remote.AdminMembershipRequest
import com.footballinsight.admin.data.remote.AdminReasonRequest
import com.footballinsight.admin.data.remote.AdminUserDto
import com.footballinsight.admin.data.remote.ServerConnectionManager
import com.footballinsight.admin.data.session.KeystoreAdminSessionStore
import com.footballinsight.admin.data.session.StoredAdminSession
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

enum class AdminDestination { Users, Audit, Settings }

data class AdminUiState(
    val authenticated: Boolean = false,
    val checkingSession: Boolean = true,
    val loading: Boolean = false,
    val destination: AdminDestination = AdminDestination.Users,
    val username: String = "",
    val displayName: String = "",
    val users: List<AdminUserDto> = emptyList(),
    val totalUsers: Long = 0,
    val selectedUser: AdminUserDto? = null,
    val auditLogs: List<AdminAuditLogDto> = emptyList(),
    val query: String = "",
    val statusFilter: String? = null,
    val tierFilter: String? = null,
    val serverUrl: String = "",
    val error: String? = null,
    val message: String? = null,
    val biometricEnabled: Boolean = false,
    val requiresBiometricUnlock: Boolean = false,
)

class AdminViewModel private constructor(context: Context) : ViewModel() {
    private val sessionStore = KeystoreAdminSessionStore(context)
    private val securityPreferences = context.getSharedPreferences("admin_local_security", Context.MODE_PRIVATE)
    private val connectionManager = ServerConnectionManager(context, sessionStore)
    private val initialBiometricEnabled = securityPreferences.getBoolean(KEY_BIOMETRIC, false)
    private val _state = MutableStateFlow(
        AdminUiState(
            serverUrl = connectionManager.baseUrl,
            biometricEnabled = initialBiometricEnabled,
            requiresBiometricUnlock = initialBiometricEnabled && sessionStore.read() != null,
        ),
    )
    val state: StateFlow<AdminUiState> = _state.asStateFlow()

    init {
        if (!_state.value.requiresBiometricUnlock) restoreSession()
    }

    fun login(username: String, password: String) = launchRequest {
        val response = connectionManager.api().login(AdminLoginRequest(username, password))
        sessionStore.save(
            StoredAdminSession(
                response.accessToken,
                response.expiresAt,
                response.admin.username,
                response.admin.displayName,
                response.admin.role,
            ),
        )
        _state.update {
            it.copy(
                authenticated = true,
                checkingSession = false,
                username = response.admin.username,
                displayName = response.admin.displayName,
            )
        }
        loadUsers()
    }

    fun navigate(destination: AdminDestination) {
        _state.update { it.copy(destination = destination, selectedUser = null, error = null) }
        if (destination == AdminDestination.Audit) loadAudit()
    }

    fun updateQuery(value: String) = _state.update { it.copy(query = value) }

    fun setFilters(status: String?, tier: String?) {
        _state.update { it.copy(statusFilter = status, tierFilter = tier) }
        loadUsers()
    }

    fun loadUsers() = launchRequest {
        val current = _state.value
        val result = connectionManager.api().users(
            query = current.query.trim().ifBlank { null },
            status = current.statusFilter,
            membershipTier = current.tierFilter,
        )
        _state.update { it.copy(users = result.items, totalUsers = result.total) }
    }

    fun openUser(id: String) = launchRequest {
        _state.update { it.copy(selectedUser = connectionManager.api().user(id)) }
    }

    fun closeUser() = _state.update { it.copy(selectedUser = null) }

    fun changeStatus(user: AdminUserDto, reason: String) = launchRequest {
        val updated = if (user.status == "active") {
            connectionManager.api().disable(user.id, AdminReasonRequest(reason))
        } else {
            connectionManager.api().restore(user.id, AdminReasonRequest(reason))
        }
        replaceUser(updated)
        _state.update { it.copy(message = if (updated.status == "active") "用户已恢复" else "用户已禁用") }
    }

    fun adjustMembership(
        user: AdminUserDto,
        tier: String,
        expirationMode: String,
        expiresAt: String?,
        reason: String,
    ) = launchRequest {
        val updated = connectionManager.api().membership(
            user.id,
            AdminMembershipRequest(tier, expirationMode, expiresAt, reason),
        )
        replaceUser(updated)
        _state.update { it.copy(message = "会员等级已调整") }
    }

    fun loadAudit() = launchRequest {
        val result = connectionManager.api().auditLogs()
        _state.update { it.copy(auditLogs = result.items) }
    }

    fun switchServer(url: String) = launchRequest {
        val normalized = connectionManager.switchTo(url)
        _state.value = AdminUiState(
            serverUrl = normalized,
            checkingSession = false,
            biometricEnabled = securityPreferences.getBoolean(KEY_BIOMETRIC, false),
            message = "服务器已切换",
        )
    }

    fun logout() = viewModelScope.launch {
        runCatching { connectionManager.api().logout() }
        sessionStore.clear()
        _state.value = AdminUiState(
            serverUrl = connectionManager.baseUrl,
            checkingSession = false,
            biometricEnabled = securityPreferences.getBoolean(KEY_BIOMETRIC, false),
        )
    }

    fun consumeNotice() = _state.update { it.copy(error = null, message = null) }

    fun biometricUnlocked(success: Boolean) {
        if (!success) {
            _state.update { it.copy(error = "生物识别未通过") }
            return
        }
        _state.update { it.copy(requiresBiometricUnlock = false) }
        restoreSession()
    }

    fun setBiometricEnabled(enabled: Boolean) {
        securityPreferences.edit().putBoolean(KEY_BIOMETRIC, enabled).apply()
        _state.update { it.copy(biometricEnabled = enabled) }
    }

    private fun restoreSession() = viewModelScope.launch {
        val stored = sessionStore.read()
        if (stored == null) {
            _state.update { it.copy(checkingSession = false) }
            return@launch
        }
        runCatching { connectionManager.api().me() }
            .onSuccess { admin ->
                _state.update {
                    it.copy(
                        authenticated = true,
                        checkingSession = false,
                        username = admin.username,
                        displayName = admin.displayName,
                    )
                }
                loadUsers()
            }
            .onFailure {
                sessionStore.clear()
                _state.update { state -> state.copy(checkingSession = false) }
            }
    }

    private fun replaceUser(updated: AdminUserDto) {
        _state.update { current ->
            current.copy(
                users = current.users.map { if (it.id == updated.id) updated else it },
                selectedUser = updated,
            )
        }
    }

    private fun launchRequest(block: suspend () -> Unit) = viewModelScope.launch {
        _state.update { it.copy(loading = true, error = null, message = null) }
        runCatching { block() }
            .onFailure { error ->
                _state.update { it.copy(error = readableError(error)) }
            }
        _state.update { it.copy(loading = false) }
    }

    private fun readableError(error: Throwable): String = when {
        error.message?.contains("401") == true -> "登录已失效，请重新登录"
        error.message?.contains("Failed to connect") == true -> "无法连接服务器，请检查服务器地址和网络"
        else -> error.message ?: "请求失败"
    }

    companion object {
        private const val KEY_BIOMETRIC = "biometric_enabled"
        fun factory(context: Context): ViewModelProvider.Factory =
            object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T =
                    AdminViewModel(context.applicationContext) as T
            }
    }
}
