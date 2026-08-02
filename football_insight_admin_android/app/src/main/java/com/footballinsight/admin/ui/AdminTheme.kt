package com.footballinsight.admin.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val AdminColors = lightColorScheme(
    primary = Color(0xFF176B52),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFD6F1E6),
    onPrimaryContainer = Color(0xFF0A3B2D),
    secondary = Color(0xFF4D5F6D),
    secondaryContainer = Color(0xFFE1E8ED),
    tertiary = Color(0xFF9B5C17),
    error = Color(0xFFB3261E),
    background = Color(0xFFF7F8F5),
    surface = Color(0xFFFDFEFB),
    surfaceVariant = Color(0xFFE8ECE7),
    outline = Color(0xFF747A75),
)

@Composable
fun AdminTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = AdminColors, content = content)
}
