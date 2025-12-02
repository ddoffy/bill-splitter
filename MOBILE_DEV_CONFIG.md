# Mobile Development Configuration

## Current Setup

The mobile apps are configured to connect to the **production server** for development:

```
Server URL: https://billsplitter.ddoffy.org
```

This allows you to develop the mobile apps independently without running the Rust backend locally.

## Benefits

✅ **Separated Development**: Front-end (mobile) and server-side development are independent  
✅ **No Local Backend Required**: Test mobile apps without running `cargo run`  
✅ **Real Data**: Use actual production data and APIs  
✅ **Faster Iteration**: Make UI changes and test immediately  
✅ **Team Collaboration**: Multiple developers can work on mobile without backend setup  

## Development Workflow

### 1. Making Changes to Mobile UI

```bash
# Edit files in static/ directory
# - static/script.js (app logic)
# - static/styles.css (styling)
# - static/index.html (structure)

# Sync changes to mobile platforms
npm run sync

# Open in IDE and rebuild
npm run open:ios      # For iOS
npm run open:android  # For Android
```

### 2. Testing

- The app will load content from `https://billsplitter.ddoffy.org`
- All API calls go to the production server
- Changes to static files require rebuild in Xcode/Android Studio

### 3. Switching to Local Server (Optional)

If you need to test against a local backend:

1. **Start your local server:**
   ```bash
   cargo run
   ```

2. **Update `capacitor.config.json`:**
   ```json
   {
     "server": {
       "url": "http://YOUR_COMPUTER_IP:7777",
       "cleartext": true
     }
   }
   ```

3. **Sync and rebuild:**
   ```bash
   npm run sync
   ```

4. **Revert when done** to production URL

### 4. Production Builds

For App Store/Google Play submission:

1. **Remove the server URL** from `capacitor.config.json`:
   ```json
   {
     "appId": "org.ddoffy.splitbills",
     "appName": "Split Bills",
     "webDir": "static"
   }
   ```

2. **Sync and build:**
   ```bash
   npm run sync
   npm run open:ios     # Build in Xcode for App Store
   npm run open:android # Build APK/AAB for Google Play
   ```

## File Structure

```
split-bills/
├── static/              # Mobile app assets
│   ├── index.html      # Entry point (loads from server)
│   ├── capacitor.js    # Capacitor initialization
│   ├── script.js       # App logic
│   └── styles.css      # Styles
├── ios/                # iOS project (Xcode)
├── android/            # Android project (Android Studio)
└── capacitor.config.json  # Points to production server
```

## Network Requirements

- **HTTPS**: Production server uses HTTPS (secure)
- **CORS**: Server must allow cross-origin requests from mobile apps
- **Connectivity**: Device/emulator must have internet access

## Troubleshooting

### App shows "Failed to Load"
- Check that `https://billsplitter.ddoffy.org` is accessible
- Verify device has internet connection
- Check server CORS configuration

### Changes not appearing
- Run `npm run sync` after editing static files
- Rebuild app in Xcode/Android Studio
- Clear app data/cache on device

### API calls failing
- Check server logs for errors
- Verify API endpoints match between mobile and server
- Check authentication/headers if required

## Configuration Reference

Current `capacitor.config.json`:
```json
{
  "appId": "org.ddoffy.splitbills",
  "appName": "Split Bills",
  "webDir": "static",
  "server": {
    "url": "https://billsplitter.ddoffy.org",
    "cleartext": false
  },
  "ios": {
    "contentInset": "automatic"
  },
  "android": {
    "allowMixedContent": false
  }
}
```

## Quick Commands

```bash
# Sync changes to mobile platforms
npm run sync

# Open in Xcode (iOS)
npm run open:ios

# Open in Android Studio (Android)
npm run open:android

# Sync specific platform only
npm run sync:ios
npm run sync:android
```

## Server-Side Development

For server-side changes, work on the main Rust codebase independently:

```bash
# Test server changes locally
cargo run

# Deploy to production
# (Mobile apps will automatically use updated server)
```

The beauty of this setup: **Server and mobile development are completely independent!**
