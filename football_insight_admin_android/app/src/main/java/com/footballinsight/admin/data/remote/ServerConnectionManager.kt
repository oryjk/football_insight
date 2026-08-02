package com.footballinsight.admin.data.remote

import android.content.Context
import com.footballinsight.admin.BuildConfig
import com.footballinsight.admin.data.session.AdminSessionStore
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class ServerConnectionManager(
    context: Context,
    private val sessionStore: AdminSessionStore,
) {
    private val preferences = context.getSharedPreferences("admin_server", Context.MODE_PRIVATE)
    private val mutex = Mutex()

    @Volatile
    var baseUrl: String = runCatching {
        normalizeServerUrl(preferences.getString(KEY_BASE_URL, BuildConfig.API_BASE_URL).orEmpty())
    }.getOrElse { normalizeServerUrl(BuildConfig.API_BASE_URL) }
        private set

    @Volatile
    private var currentApi = ApiFactory.create(sessionStore, baseUrl)

    fun api(): AdminApi = currentApi

    suspend fun switchTo(rawUrl: String): String = mutex.withLock {
        val normalized = normalizeServerUrl(rawUrl)
        val candidate = ApiFactory.create(sessionStore, normalized)
        val response = runCatching { candidate.probe() }
            .getOrElse { throw IllegalStateException("无法连接服务器：${it.message ?: "网络错误"}") }
        check(response.isSuccessful) { "服务器验证失败（HTTP ${response.code()}）" }
        check(response.headers()["Content-Type"]?.contains("application/json") == true) {
            "服务器验证失败：返回内容不是 Football Insight API"
        }
        check(response.body()?.wechatLoginEnabled != null) {
            "服务器验证失败：缺少 Football Insight API 标识"
        }
        if (normalized != baseUrl) {
            check(preferences.edit().putString(KEY_BASE_URL, normalized).commit()) { "服务器地址保存失败" }
            baseUrl = normalized
            currentApi = candidate
            sessionStore.clear()
        }
        normalized
    }

    private companion object {
        const val KEY_BASE_URL = "base_url"
    }
}
