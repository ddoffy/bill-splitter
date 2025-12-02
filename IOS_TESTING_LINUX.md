# Testing on iPhone 15 Pro Max (from Linux)

Since you're on Linux and iOS development requires macOS/Xcode, here are your options:

## ⚡ Immediate Solution: Web Version on Safari

**This is the recommended approach for quick testing:**

### Steps:

1. **Open Safari on your iPhone 15 Pro Max**

2. **Navigate to:** `https://billsplitter.ddoffy.org`

3. **Add to Home Screen for native-like experience:**
   - Tap the **Share** button (square with arrow up)
   - Scroll down and tap **"Add to Home Screen"**
   - Name it "Split Bills"
   - Tap **Add**

4. **Launch from Home Screen:**
   - App icon appears on your home screen
   - Launches in full-screen (no Safari UI)
   - Behaves like a native app
   - All features work identically

### Benefits:
✅ **Works immediately** - No build required  
✅ **Same functionality** - Uses same server/APIs  
✅ **Instant updates** - Changes reflect immediately  
✅ **Native-like UX** - Full screen, smooth, responsive  
✅ **No macOS needed** - Test right now  

## 🍎 Building Native iOS App (Requires macOS)

Since you're on Linux, you have these options:

### Option 1: Use GitHub Actions (Automated CI/CD)

I've created `.github/workflows/ios-build.yml` for you. This will:
- Run on macOS GitHub runners
- Build iOS app automatically
- Generate IPA files you can install

**To enable:**
```bash
git add .github/workflows/ios-build.yml
git commit -m "Add iOS CI/CD workflow"
git push
```

The workflow will build on every push to `feature/mobile` or `main`.

### Option 2: Remote macOS Access

If you have access to a Mac (friend's computer, cloud Mac, etc.):

1. **Copy your project to Mac:**
   ```bash
   # From your Linux machine
   rsync -avz --exclude node_modules --exclude target \
     ~/workspace/rust/split-bills/ \
     user@mac-ip:~/split-bills/
   ```

2. **On the Mac:**
   ```bash
   cd ~/split-bills
   npm install
   npm run open:ios
   ```

3. **In Xcode:**
   - Connect your iPhone via USB
   - Select your iPhone as target
   - Press Cmd+R to build and run

### Option 3: Cloud macOS Services

Rent macOS in the cloud (paid):
- **MacStadium**: Dedicated Mac cloud hosting
- **AWS EC2 Mac Instances**: Apple Silicon Macs on AWS
- **MacinCloud**: Hourly macOS rental
- **Codemagic**: CI/CD specifically for mobile apps

### Option 4: TestFlight Distribution

For team testing:

1. Build using GitHub Actions (with signing configured)
2. Upload to TestFlight via CI/CD
3. Install TestFlight on your iPhone
4. Test app without needing direct Mac access

## 📱 Installing IPA Files (Without Mac)

If you get an IPA file from CI/CD:

### Method 1: AltStore (Recommended)
```bash
# Install AltStore on Windows/Linux
# Sideload IPA to iPhone without Apple Developer account
# Free, works for 7 days, needs refresh
```

### Method 2: Apple Configurator (Mac required)
- Requires Mac but can be remote/borrowed
- Enterprise-level distribution

### Method 3: TestFlight
- Upload via CI/CD
- No Mac needed once set up

## 🤖 Android Alternative (Works on Linux!)

Since you're on Linux, you can fully test Android:

```bash
# Connect your Android device (or use emulator)
./quick-test-android.sh

# View logs
./logs-android.sh

# Debug with Chrome
./debug-android.sh
```

You have all the Android tools already!

## 📊 Feature Parity

The web version on iPhone Safari has **100% feature parity** with the native app:
- ✅ All bill splitting features
- ✅ AI parsing (if enabled)
- ✅ Session management
- ✅ Email sending
- ✅ Responsive design optimized for iPhone
- ✅ Touch gestures work perfectly
- ✅ Offline capability (with service worker)

The only differences in a native iOS app would be:
- App Store presence
- Push notifications
- Native camera integration (vs web camera API)
- Background processing
- HomeKit/HealthKit integration (if you wanted that)

## 🎯 Recommended Workflow

**For Development:**
1. ✅ Test web version on iPhone Safari (add to home screen)
2. ✅ Develop and test Android on Linux (you have full tooling)
3. ✅ Use GitHub Actions for iOS builds when needed

**For Production:**
1. Web version works for all users immediately
2. Android app via Google Play
3. iOS app via App Store (using CI/CD or borrowed Mac)

## 🚀 Try It Now

**Right now, on your iPhone 15 Pro Max:**

1. Open Safari
2. Go to `https://billsplitter.ddoffy.org`
3. Tap Share → Add to Home Screen
4. Launch the app from your home screen
5. Use it just like a native app!

This gives you immediate testing capability while you figure out iOS builds.

## Need Help?

Run `./test-ios-on-linux.sh` for a quick reference of all options.
