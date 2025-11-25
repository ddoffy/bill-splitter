# Bill Splitter

A web application built with Rust and Axum for splitting bills after a party.

## Features

- ✅ Add people with their names and amounts spent
- ✅ Mark people as sponsors
- ✅ Toggle to include/exclude sponsors in the split calculation
- ✅ Calculate who should pay or receive refunds
- ✅ Beautiful, responsive web interface
- ✅ RESTful API backend with Axum
- ✅ Server-side rendering with Askama templates

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Templating**: Askama
- **Frontend**: Vanilla JavaScript with Fetch API
- **Styling**: Custom CSS

## Project Structure

```
split-bills/
├── Cargo.toml              # Rust dependencies
├── src/
│   └── main.rs            # Axum server and API endpoints
├── templates/
│   └── index.html         # Main HTML template
└── static/
    ├── styles.css         # Stylesheet
    └── script.js          # Frontend JavaScript
```

## How to Run

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
