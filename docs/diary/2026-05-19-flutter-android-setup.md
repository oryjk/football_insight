# Flutter 开发环境搭建日记

## 日期：2026-05-19

## 背景

为 Football Insight 开发 Flutter 移动端 App（球票回流监控 + JPush 推送），首次在本地搭建 Flutter Android 开发环境。

## 环境信息

- macOS (Apple Silicon)
- Flutter 3.41.6 (stable channel)
- Android Emulator: Medium_Phone_API_36 (Android 16, arm64)

## 踩坑记录

### 1. Android 模拟器启动报错 "exited with code 1"

**错误：**
```
The Android emulator exited with code 1 during startup
Android emulator stderr:
Address these issues and try again.
```

**排查：** 用 verbose 模式启动查看详细日志：
```bash
~/Library/Android/sdk/emulator/emulator -avd Medium_Phone_API_36 -verbose
```

**原因：** 已有一个同名模拟器实例在运行。

**解决：**
```bash
# 方法1：直接用已有的模拟器
flutter devices   # 确认设备已连接
flutter run       # 直接跑

# 方法2：杀掉重开
killall qemu-system-aarch64 2>/dev/null; killall emulator 2>/dev/null
flutter emulators --launch Medium_Phone_API_36
```

### 2. Gradle 依赖下载卡死

**现象：** `flutter run` 卡在 `Running Gradle task 'assembleDebug'...`，一直转圈。

**排查：** 加 `--verbose` 看详细输出：
```bash
flutter run --verbose
```

发现卡在从 `plugins.gradle.org` 下载 Kotlin 插件，国内直连很慢。

### 3. 配置阿里云镜像加速

#### 项目级配置

修改 `android/settings.gradle.kts` — pluginManagement repositories：
```kotlin
repositories {
    maven { url = uri("https://maven.aliyun.com/repository/google") }
    maven { url = uri("https://maven.aliyun.com/repository/central") }
    maven { url = uri("https://maven.aliyun.com/repository/gradle-plugin") }
    google()
    mavenCentral()
    gradlePluginPortal()
}
```

修改 `android/build.gradle.kts` — allprojects repositories：
```kotlin
repositories {
    maven { url = uri("https://maven.aliyun.com/repository/google") }
    maven { url = uri("https://maven.aliyun.com/repository/central") }
    google()
    mavenCentral()
}
```

修改 `android/gradle/wrapper/gradle-wrapper.properties` — Gradle 发行版下载：
```properties
distributionUrl=https\://mirrors.cloud.tencent.com/gradle/gradle-8.14-all.zip
```

#### 全局配置（关键！）

项目级配置只对 Maven 仓库生效，Flutter SDK 内部的 Kotlin 插件下载走的是 `plugins.gradle.org`，项目级配置管不到。

需要配置全局 `init.gradle.kts`：

```bash
mkdir -p ~/.gradle && cat > ~/.gradle/init.gradle.kts << 'EOF'
settingsEvaluated {
    pluginManagement {
        repositories {
            maven { url = uri("https://maven.aliyun.com/repository/gradle-plugin") }
            maven { url = uri("https://maven.aliyun.com/repository/google") }
            maven { url = uri("https://maven.aliyun.com/repository/central") }
            gradlePluginPortal()
            google()
            mavenCentral()
        }
    }
    dependencyResolutionManagement {
        repositories {
            maven { url = uri("https://maven.aliyun.com/repository/google") }
            maven { url = uri("https://maven.aliyun.com/repository/central") }
            maven { url = uri("https://maven.aliyun.com/repository/gradle-plugin") }
            google()
            mavenCentral()
        }
    }
}
EOF
```

**这个配置对所有 Gradle 项目生效，包括 Flutter SDK 内部的依赖下载。配置后立刻生效。**

## 验证结果

```bash
# 静态分析
cd football_insight_app && flutter analyze
# => No issues found!

# 单元测试
flutter test
# => 00:00 +11: All tests passed!

# Android 真机运行
flutter run
# => 编译成功，App 在模拟器上运行
```

## 经验总结

1. **国内 Flutter/Android 开发必须配镜像**，否则 Gradle 下载依赖会卡死
2. **项目级配置不够**，必须配全局 `~/.gradle/init.gradle.kts` 才能覆盖 Flutter SDK 内部依赖
3. **Android 模拟器报错先看 verbose 日志**，不要瞎猜
4. **第一次编译很慢**（下载依赖 + 编译），后续增量编译很快
5. `flutter run --verbose` 是排查卡住问题的首选工具
