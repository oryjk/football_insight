package com.footballinsight.admin.data.remote

import com.footballinsight.admin.data.session.InMemoryAdminSessionStore
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Request
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class NetworkConfigurationTest {
    @Test
    fun normalizes_server_urls_for_retrofit() {
        assertEquals("http://10.0.2.2:8080/", normalizeServerUrl("10.0.2.2:8080"))
        assertEquals(
            "https://match.oryjk.cn/",
            normalizeServerUrl(" https://match.oryjk.cn "),
        )
        assertThrows(IllegalArgumentException::class.java) {
            normalizeServerUrl("https://user:pass@match.oryjk.cn")
        }
    }

    @Test
    fun adds_admin_bearer_token_to_authenticated_requests() = runBlocking {
        val store = InMemoryAdminSessionStore("admin-jwt")
        val client = OkHttpClient.Builder()
            .addInterceptor(AdminBearerInterceptor(store))
            .addInterceptor { chain ->
                assertEquals("Bearer admin-jwt", chain.request().header("Authorization"))
                okhttp3.Response.Builder()
                    .request(chain.request())
                    .protocol(okhttp3.Protocol.HTTP_1_1)
                    .code(204)
                    .message("No Content")
                    .body(okhttp3.ResponseBody.create(null, ByteArray(0)))
                    .build()
            }
            .build()

        client.newCall(Request.Builder().url("https://match.oryjk.cn/api/v1/admin/users").build())
            .execute()
            .close()
    }
}
