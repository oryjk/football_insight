package com.footballinsight.admin.data.session

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

data class StoredAdminSession(
    val accessToken: String,
    val expiresAt: String = "",
    val username: String = "",
    val displayName: String = "",
    val role: String = "",
)

interface AdminSessionStore {
    fun read(): StoredAdminSession?
    fun save(session: StoredAdminSession)
    fun clear()
}

class KeystoreAdminSessionStore(context: Context) : AdminSessionStore {
    private val preferences = EncryptedSharedPreferences.create(
        context,
        "admin_secure_session",
        MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build(),
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    override fun read(): StoredAdminSession? {
        val token = preferences.getString(KEY_TOKEN, null)?.takeIf(String::isNotBlank) ?: return null
        return StoredAdminSession(
            accessToken = token,
            expiresAt = preferences.getString(KEY_EXPIRES_AT, "").orEmpty(),
            username = preferences.getString(KEY_USERNAME, "").orEmpty(),
            displayName = preferences.getString(KEY_DISPLAY_NAME, "").orEmpty(),
            role = preferences.getString(KEY_ROLE, "").orEmpty(),
        )
    }

    override fun save(session: StoredAdminSession) {
        preferences.edit()
            .putString(KEY_TOKEN, session.accessToken)
            .putString(KEY_EXPIRES_AT, session.expiresAt)
            .putString(KEY_USERNAME, session.username)
            .putString(KEY_DISPLAY_NAME, session.displayName)
            .putString(KEY_ROLE, session.role)
            .commit()
    }

    override fun clear() {
        preferences.edit().clear().commit()
    }

    private companion object {
        const val KEY_TOKEN = "access_token"
        const val KEY_EXPIRES_AT = "expires_at"
        const val KEY_USERNAME = "username"
        const val KEY_DISPLAY_NAME = "display_name"
        const val KEY_ROLE = "role"
    }
}

class InMemoryAdminSessionStore(token: String? = null) : AdminSessionStore {
    private var session = token?.let(::StoredAdminSession)

    override fun read(): StoredAdminSession? = session
    override fun save(session: StoredAdminSession) { this.session = session }
    override fun clear() { session = null }
}
