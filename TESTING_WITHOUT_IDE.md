# Testing Mobile Apps Without IDE

You can test and build the mobile apps without Android Studio or Xcode using command-line tools!

## Android Testing (Without Android Studio)

### Prerequisites

```bash
# Install Java (if not already installed)
sudo apt install openjdk-17-jdk

# Verify installation
java -version
adb version
```

### Quick Start

1. **Connect a device or start emulator:**
   ```bash
   # Option 1: Connect physical device via USB
   # - Enable Developer Options on your Android device
   # - Enable USB Debugging
   # - Connect via USB
   
   # Option 2: Use wireless debugging
   adb connect <device-ip>:5555
   
   # Verify device is connected
   adb devices
   ```

2. **Build and install:**
   ```bash
   # Quick test (sync + build + install)
   ./quick-test-android.sh
   
   # Or step by step:
   npm run sync:android          # Sync web assets
   npm run android:build         # Build APK
   npm run android:install       # Install on device
   ```

3. **View logs:**
   ```bash
   ./logs-android.sh
   # or
   npm run android:logs
   ```

### Available Scripts

```bash
# Build debug APK
./test-android.sh
npm run test:android

# Quick test (sync + build + install)
./quick-test-android.sh
npm run test:quick

# View real-time logs
./logs-android.sh
npm run android:logs

# Build only (no install)
npm run android:build

# Build release APK
npm run android:release

# Install APK to connected device
npm run android:install
```

### Manual APK Installation

After building, the APK is located at:
```
android/app/build/outputs/apk/debug/app-debug.apk
```

Install it:
```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

### Testing Options

#### 1. Physical Device (Recommended)
- **Pros**: Real device performance, actual touch input, camera, sensors
- **Cons**: Requires USB cable or wireless setup
- **Setup**:
  ```bash
  # Enable USB debugging on your phone
  # Connect via USB
  adb devices  # Should show your device
  ```

#### 2. Android Emulator
- **Pros**: No physical device needed
- **Cons**: Slower, requires more RAM, no real sensors
- **Setup**:
  ```bash
  # Install Android emulator
  sudo apt install android-emulator
  
  # Or use Android Studio's AVD Manager once
  # Then close Android Studio and use CLI
  ```

#### 3. Wireless Debugging (Android 11+)
- **Pros**: No USB cable needed
- **Cons**: Device and computer must be on same network
- **Setup**:
  ```bash
  # On your phone: Settings → Developer Options → Wireless Debugging
  # Note the IP and port
  adb pair <ip>:<pairing-port>  # Enter pairing code
  adb connect <ip>:<port>
  ```

### Development Workflow

```bash
# 1. Make changes to static files
vim static/script.js

# 2. Quick test (one command)
./quick-test-android.sh

# 3. View logs in another terminal
./logs-android.sh
```

### Debugging

**View Chrome DevTools:**
1. Build and install app
2. Open Chrome on your computer
3. Navigate to: `chrome://inspect`
4. Click "inspect" under your device
5. Full Chrome DevTools with console, network, etc.

**Common Issues:**

```bash
# Device not found
adb devices           # Check connection
adb kill-server       # Restart adb
adb start-server

# Permission denied
adb shell             # Should open shell
# If fails, re-enable USB debugging on phone

# Build fails
cd android
./gradlew clean       # Clean build cache
./gradlew assembleDebug
```

## iOS Testing (Without Xcode) - macOS Only

For iOS, you **need** Xcode for initial setup, but can use CLI for builds:

```bash
# Build from command line (after opening in Xcode once)
cd ios/App
xcodebuild -workspace App.xcworkspace \
           -scheme App \
           -configuration Debug \
           -destination 'platform=iOS Simulator,name=iPhone 15' \
           clean build
```

For iOS, it's **highly recommended** to use Xcode as Apple's tooling is tightly integrated.

## Browser Testing (Web Version)

You can also test the web version directly:

```bash
# Start the backend (if testing locally)
cargo run

# Or test against production
# Just open static/index.html in a browser
# It will redirect to the server
```

## Continuous Integration

For automated testing in CI/CD:

```bash
# In your CI pipeline
cd android
./gradlew assembleDebug
./gradlew test

# APK is at: app/build/outputs/apk/debug/app-debug.apk
```

## Production Builds

### Android Release APK/AAB

```bash
# Sync latest changes
npm run sync:android

# Build release
cd android
./gradlew assembleRelease  # APK
./gradlew bundleRelease    # AAB for Google Play

# Sign the APK
# (You'll need a keystore - see Android docs)
```

### iOS Release Build

iOS requires Xcode for App Store builds:
```bash
npm run open:ios
# Then: Product → Archive in Xcode
```

## Summary

| Task | Command |
|------|---------|
| Quick test Android | `./quick-test-android.sh` |
| Build Android APK | `npm run android:build` |
| Install on device | `npm run android:install` |
| View logs | `./logs-android.sh` |
| Chrome DevTools | `chrome://inspect` |
| Open Android Studio | `npm run open:android` |
| Open Xcode | `npm run open:ios` |

**Recommendation**: Use CLI for quick iterations, use IDE for complex debugging or first-time setup.
