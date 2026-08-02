package com.footballinsight.admin.data.remote

import com.footballinsight.admin.BuildConfig
import com.footballinsight.admin.data.session.AdminSessionStore
import com.google.gson.FieldNamingPolicy
import com.google.gson.GsonBuilder
import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.util.concurrent.TimeUnit

class AdminBearerInterceptor(
    private val sessionStore: AdminSessionStore,
) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val token = sessionStore.read()?.accessToken
        val request = if (token.isNullOrBlank()) {
            chain.request()
        } else {
            chain.request().newBuilder()
                .header("Authorization", "Bearer $token")
                .build()
        }
        return chain.proceed(request)
    }
}

object ApiFactory {
    fun create(
        sessionStore: AdminSessionStore,
        baseUrl: String = BuildConfig.API_BASE_URL,
    ): AdminApi {
        val client = OkHttpClient.Builder()
            .addInterceptor(AdminBearerInterceptor(sessionStore))
            .connectTimeout(10, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .retryOnConnectionFailure(true)
            .build()
        return Retrofit.Builder()
            .baseUrl(normalizeServerUrl(baseUrl))
            .client(client)
            .addConverterFactory(
                GsonConverterFactory.create(
                    GsonBuilder()
                        .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
                        .create(),
                ),
            )
            .build()
            .create(AdminApi::class.java)
    }
}

fun normalizeServerUrl(value: String): String {
    val trimmed = value.trim().ifBlank { BuildConfig.API_BASE_URL }
    val withScheme = if ("://" in trimmed) trimmed else "http://$trimmed"
    val parsed = withScheme.toHttpUrlOrNull() ?: throw IllegalArgumentException("服务器地址格式无效")
    require(parsed.scheme == "http" || parsed.scheme == "https") { "仅支持 HTTP 或 HTTPS" }
    require(parsed.username.isEmpty() && parsed.password.isEmpty()) { "服务器地址不能包含账号信息" }
    require(parsed.query == null && parsed.fragment == null) { "服务器地址不能包含查询参数或锚点" }
    return parsed.newBuilder().encodedPath(parsed.encodedPath.trimEnd('/') + "/").build().toString()
}
