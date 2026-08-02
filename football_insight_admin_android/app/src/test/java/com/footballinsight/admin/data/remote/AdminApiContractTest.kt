package com.footballinsight.admin.data.remote

import com.footballinsight.admin.data.session.InMemoryAdminSessionStore
import kotlinx.coroutines.runBlocking
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class AdminApiContractTest {
    private lateinit var server: MockWebServer
    private lateinit var api: AdminApi

    @Before
    fun setUp() {
        server = MockWebServer().also { it.start() }
        api = ApiFactory.create(InMemoryAdminSessionStore("admin-token"), server.url("/").toString())
    }

    @After
    fun tearDown() = server.shutdown()

    @Test
    fun membership_request_uses_admin_path_bearer_token_and_snake_case() = runBlocking {
        server.enqueue(
            MockResponse()
                .setHeader("Content-Type", "application/json")
                .setBody(
                    """{"id":"u1","account_identifier":"fan","display_name":"球迷","avatar_url":null,"has_wechat_binding":false,"status":"active","invite_code":null,"invited_by":null,"membership_tier":"V8","membership_expires_at":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}""",
                ),
        )

        val result = api.membership(
            "u1",
            AdminMembershipRequest("V8", "never", null, "线下年度会员"),
        )
        val request = server.takeRequest()

        assertEquals("/api/v1/admin/users/u1/membership", request.path)
        assertEquals("Bearer admin-token", request.getHeader("Authorization"))
        assertTrue(request.body.readUtf8().contains("\"membership_tier\":\"V8\""))
        assertEquals("V8", result.membershipTier)
        assertEquals("球迷", result.displayName)
    }

    @Test
    fun server_probe_uses_versioned_json_api_instead_of_health_fallback() = runBlocking {
        server.enqueue(
            MockResponse()
                .setHeader("Content-Type", "application/json")
                .setBody("""{"wechat_login_enabled":false}"""),
        )

        val response = api.probe()
        val request = server.takeRequest()

        assertEquals("/api/v1/system/public-config", request.path)
        assertEquals(false, response.body()?.wechatLoginEnabled)
    }
}
