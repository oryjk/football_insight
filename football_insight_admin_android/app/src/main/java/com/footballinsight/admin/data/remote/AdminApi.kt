package com.footballinsight.admin.data.remote

import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query

data class AdminLoginRequest(val username: String, val password: String)
data class AdminIdentityDto(
    val id: String,
    val username: String,
    val displayName: String,
    val role: String,
)
data class AdminAuthResponseDto(
    val accessToken: String,
    val expiresAt: String,
    val admin: AdminIdentityDto,
)
data class AdminInviterDto(
    val id: String,
    val displayName: String,
    val accountIdentifier: String,
    val referralInviteCode: String,
)
data class AdminUserDto(
    val id: String,
    val accountIdentifier: String,
    val displayName: String,
    val avatarUrl: String?,
    val hasWechatBinding: Boolean,
    val status: String,
    val inviteCode: String?,
    val invitedBy: AdminInviterDto?,
    val membershipTier: String,
    val membershipExpiresAt: String?,
    val createdAt: String,
    val updatedAt: String,
    val referrals: List<AdminReferredUserDto>? = null,
    val activity: AdminUserActivityDto? = null,
    val orders: List<AdminPaymentOrderDto>? = null,
    val subscriptions: List<AdminSubscriptionDto>? = null,
    val devices: List<AdminUserDeviceDto>? = null,
)
data class AdminReferredUserDto(
    val id: String,
    val accountIdentifier: String,
    val displayName: String,
    val status: String,
    val createdAt: String,
)
data class AdminUserActivityDto(
    val lastLoginAt: String?,
    val lastActiveAt: String?,
    val lastActivePageKey: String?,
)
data class AdminPaymentOrderDto(
    val orderNo: String,
    val amountCents: Int,
    val status: String,
    val productType: String,
    val paidAt: String?,
    val createdAt: String,
)
data class AdminSubscriptionDto(
    val id: String,
    val planCode: String,
    val scope: String,
    val teamCode: String,
    val season: Int?,
    val matchId: Long?,
    val status: String,
    val startsAt: String,
    val expiresAt: String?,
)
data class AdminUserDeviceDto(
    val id: Long,
    val platform: String,
    val maskedDeviceToken: String,
    val createdAt: String,
    val updatedAt: String,
)
data class AdminUserPageDto(
    val total: Long,
    val page: Long,
    val pageSize: Long,
    val items: List<AdminUserDto>,
)
data class AdminReasonRequest(val reason: String)
data class AdminMembershipRequest(
    val membershipTier: String,
    val expirationMode: String,
    val membershipExpiresAt: String? = null,
    val reason: String,
)
data class AdminAuditLogDto(
    val id: String,
    val adminUsername: String,
    val action: String,
    val targetType: String,
    val targetId: String?,
    val reason: String?,
    val createdAt: String,
)
data class AdminAuditPageDto(
    val total: Long,
    val page: Long,
    val pageSize: Long,
    val items: List<AdminAuditLogDto>,
)

interface AdminApi {
    @GET("api/v1/system/public-config")
    suspend fun probe(): Response<PublicConfigProbe>

    @POST("api/v1/admin/auth/login")
    suspend fun login(@Body request: AdminLoginRequest): AdminAuthResponseDto

    @GET("api/v1/admin/auth/me")
    suspend fun me(): AdminIdentityDto

    @POST("api/v1/admin/auth/logout")
    suspend fun logout(): Response<Unit>

    @GET("api/v1/admin/users")
    suspend fun users(
        @Query("query") query: String? = null,
        @Query("status") status: String? = null,
        @Query("membership_tier") membershipTier: String? = null,
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = 50,
    ): AdminUserPageDto

    @GET("api/v1/admin/users/{id}")
    suspend fun user(@Path("id") id: String): AdminUserDto

    @POST("api/v1/admin/users/{id}/disable")
    suspend fun disable(@Path("id") id: String, @Body request: AdminReasonRequest): AdminUserDto

    @POST("api/v1/admin/users/{id}/restore")
    suspend fun restore(@Path("id") id: String, @Body request: AdminReasonRequest): AdminUserDto

    @POST("api/v1/admin/users/{id}/membership")
    suspend fun membership(
        @Path("id") id: String,
        @Body request: AdminMembershipRequest,
    ): AdminUserDto

    @GET("api/v1/admin/audit-logs")
    suspend fun auditLogs(
        @Query("page") page: Int = 1,
        @Query("page_size") pageSize: Int = 50,
    ): AdminAuditPageDto
}

data class PublicConfigProbe(val wechatLoginEnabled: Boolean?)
