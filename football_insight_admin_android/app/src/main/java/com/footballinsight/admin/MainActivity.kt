package com.footballinsight.admin

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.footballinsight.admin.ui.AdminApp
import com.footballinsight.admin.ui.AdminTheme
import com.footballinsight.admin.ui.AdminViewModel

class MainActivity : FragmentActivity() {
    private val viewModel: AdminViewModel by viewModels { AdminViewModel.factory(this) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AdminTheme { AdminApp(viewModel, ::requestBiometricUnlock) }
        }
    }

    private fun requestBiometricUnlock() {
        val prompt = BiometricPrompt(
            this,
            ContextCompat.getMainExecutor(this),
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    viewModel.biometricUnlocked(true)
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    viewModel.biometricUnlocked(false)
                }
            },
        )
        prompt.authenticate(
            BiometricPrompt.PromptInfo.Builder()
                .setTitle("解锁足球洞察管理")
                .setSubtitle("验证本机管理员身份")
                .setNegativeButtonText("取消")
                .build(),
        )
    }
}
