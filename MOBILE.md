# Mobile App Development Guide

This guide explains how to build and run the Split Bills mobile apps for iOS and Android.

## Prerequisites

### For iOS Development:
- macOS computer
- Xcode (latest version recommended)
- CocoaPods: `sudo gem install cocoapods`
- iOS device or simulator

### For Android Development:
- Android Studio
- Java Development Kit (JDK) 17 or newer
- Android SDK
- Android device or emulator

## Project Structure

```
split-bills/
├── static/              # Web assets (HTML, CSS, JS)
│   ├── index.html      # Mobile app entry point
│   ├── capacitor.js    # Capacitor initialization
│   ├── script.js       # Main app logic
│   └── styles.css      # App styles
├── ios/                # iOS native project (generated)
├── android/            # Android native project (generated)
└── capacitor.config.json  # Capacitor configuration
```

## Setup

### 1. Install Dependencies

```bash
npm install
```

### 2. Sync Web Assets

After making changes to the web code in `static/`:

```bash
npx cap sync
```

This command:
- Copies web assets to native projects
- Updates native dependencies
- Syncs configuration

## Development

### Running the Backend Server

The mobile app needs the Rust backend running:

```bash
cargo run
```

The server will start on `http://localhost:8080`.

### iOS Development

1. **Open the project in Xcode:**
   ```bash
   npx cap open ios
   ```

2. **Configure the app in Xcode:**
   - Select your development team in "Signing & Capabilities"
   - Choose your target device (simulator or connected iPhone)

3. **Run the app:**
   - Click the "Play" button in Xcode, or press `Cmd+R`
   - The app will build and launch on your selected device

4. **Live Reload (Optional):**
   - Make sure your Mac and iPhone are on the same network
   - Update `capacitor.config.json` with your Mac's IP:
     ```json
     "server": {
       "url": "http://192.168.1.xxx:8080"
     }
     ```
   - Run `npx cap sync ios`
   - The app will now load content from your development server

### Android Development

1. **Open the project in Android Studio:**
   ```bash
   npx cap open android
   ```

2. **Wait for Gradle sync to complete**

3. **Run the app:**
   - Select your target device (emulator or connected phone)
   - Click the "Run" button or press `Shift+F10`

4. **Live Reload (Optional):**
   - Make sure your computer and Android device are on the same network
   - Update `capacitor.config.json` with your computer's IP:
     ```json
     "server": {
       "url": "http://192.168.1.xxx:8080"
     }
     ```
   - Run `npx cap sync android`
   - The app will now load content from your development server

## Building for Production

### iOS

1. **Remove development server config:**
   - Edit `capacitor.config.json` and remove/comment out the `server` section
   - Run `npx cap sync ios`

2. **Build in Xcode:**
   - Open the project: `npx cap open ios`
   - Select "Any iOS Device (arm64)" as the build target
   - Product → Archive
   - Follow Apple's TestFlight/App Store submission process

### Android

1. **Remove development server config:**
   - Edit `capacitor.config.json` and remove/comment out the `server` section
   - Run `npx cap sync android`

2. **Build release APK/AAB:**
   ```bash
   cd android
   ./gradlew assembleRelease  # For APK
   ./gradlew bundleRelease    # For AAB (Google Play)
   ```

3. **Sign and distribute:**
   - The unsigned APK will be at `android/app/build/outputs/apk/release/`
   - Sign it using Android Studio or `jarsigner`
   - Upload to Google Play Console

## Troubleshooting

### iOS Issues

**CocoaPods not found:**
```bash
sudo gem install cocoapods
cd ios/App
pod install
```

**Build failed - "No code signing identities found":**
- Open Xcode
- Go to Preferences → Accounts
- Add your Apple ID
- Select your team in the project settings

### Android Issues

**Gradle sync failed:**
- Make sure you have JDK 17 installed
- Update Android Studio to the latest version
- Run `./gradlew clean` in the `android` directory

**App crashes on startup:**
- Check logcat in Android Studio
- Verify the backend server is running
- Check network permissions in AndroidManifest.xml

### General Issues

**"Failed to load content" error:**
- Make sure the Rust backend is running
- Check that the server URL in `capacitor.config.json` is correct
- Verify network connectivity
- Check the browser console for errors

**Web assets not updating:**
```bash
npx cap sync
```

## Useful Commands

```bash
# Sync all platforms
npx cap sync

# Sync specific platform
npx cap sync ios
npx cap sync android

# Open in IDE
npx cap open ios
npx cap open android

# Copy web assets only
npx cap copy

# Update native dependencies
npx cap update

# List installed plugins
npx cap ls
```

## Adding Native Features

To add native capabilities (camera, filesystem, etc.):

1. **Install the plugin:**
   ```bash
   npm install @capacitor/camera @capacitor/filesystem
   ```

2. **Sync to native projects:**
   ```bash
   npx cap sync
   ```

3. **Use in your code:**
   ```javascript
   import { Camera } from '@capacitor/camera';
   
   const photo = await Camera.getPhoto({
     quality: 90,
     allowEditing: true,
     resultType: 'uri'
   });
   ```

## Resources

- [Capacitor Documentation](https://capacitorjs.com/docs)
- [Capacitor iOS Guide](https://capacitorjs.com/docs/ios)
- [Capacitor Android Guide](https://capacitorjs.com/docs/android)
- [Capacitor Plugins](https://capacitorjs.com/docs/plugins)
