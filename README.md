# Bill Splitter

A web application built with Rust and Axum for splitting bills after a party. **Now with native iOS and Android support!** 📱

## Features

- ✅ Add people with their names and amounts spent
- ✅ Mark people as sponsors
- ✅ Toggle to include/exclude sponsors in the split calculation
- ✅ Calculate who should pay or receive refunds
- ✅ Beautiful, responsive web interface
- ✅ RESTful API backend with Axum
- ✅ Server-side rendering with Askama templates
- 🆕 **Native iOS app** - Build and run on iPhone/iPad
- 🆕 **Native Android app** - Build and run on Android devices

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Templating**: Askama
- **Frontend**: Vanilla JavaScript with Fetch API
- **Mobile**: Capacitor (native iOS/Android apps)
- **Styling**: Custom CSS

## Project Structure

```
split-bills/
├── Cargo.toml              # Rust dependencies
├── package.json            # Node.js dependencies for mobile
├── capacitor.config.json   # Mobile app configuration
├── src/
│   └── main.rs            # Axum server and API endpoints
├── templates/
│   └── index.html         # Main HTML template
├── static/
│   ├── styles.css         # Stylesheet
│   ├── script.js          # Frontend JavaScript
│   └── index.html         # Mobile app entry point
├── ios/                    # iOS native project (generated)
└── android/                # Android native project (generated)
```

## How to Run

### Web Version

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build and run the project**:
   ```bash
   cargo run
   ```

3. **Open your browser** and navigate to:
   ```
   http://127.0.0.1:8080
   ```

### Mobile Apps (iOS/Android)

For detailed instructions on building and running the native mobile apps, see **[MOBILE.md](MOBILE.md)**.

Quick start:
```bash
# Run the setup script
./setup-mobile.sh

# Start the backend
cargo run

# Open in Xcode (iOS)
npm run open:ios

# Or open in Android Studio (Android)
npm run open:android
```

## Usage

1. **Add People**: Enter each person's name, amount they spent, and check if they're a sponsor
2. **Configure Split**: Choose whether to include sponsors in the equal split
3. **Calculate**: Click "Calculate Split" to see who owes money or gets refunded

### Calculation Modes

- **Include sponsors in split**: Everyone shares the total equally, regardless of sponsor status
- **Exclude sponsors from split**: Sponsors get back what they spent, non-sponsors split the total

## API Endpoints

- `GET /` - Serve the main web page
- `GET /api/people` - Get all people
- `POST /api/people` - Add a new person
- `DELETE /api/people/:id` - Remove a person
- `POST /api/calculate` - Calculate the bill split

## Example

If Alice spent $100, Bob spent $50, and Charlie spent $0:
- **Total**: $150
- **Per person**: $50 (if all 3 participate)
- **Result**: 
  - Alice should receive $50
  - Bob is settled
  - Charlie should pay $50

## License

MIT
