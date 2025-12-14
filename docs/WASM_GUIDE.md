# WebAssembly (WASM) Implementation Guide

This document explains how WebAssembly is implemented in the Split Bills project for offline calculation support, and provides a deep understanding of WASM in Rust.

## Table of Contents

1. [What is WebAssembly?](#what-is-webassembly)
2. [Why WASM for This Project?](#why-wasm-for-this-project)
3. [Architecture Overview](#architecture-overview)
4. [Core Concepts of WASM in Rust](#core-concepts-of-wasm-in-rust)
5. [Project Structure](#project-structure)
6. [Implementation Details](#implementation-details)
7. [Building WASM](#building-wasm)
8. [JavaScript Integration](#javascript-integration)
9. [Memory Management](#memory-management)
10. [Best Practices](#best-practices)
11. [Troubleshooting](#troubleshooting)

---

## What is WebAssembly?

WebAssembly (WASM) is a binary instruction format designed as a portable compilation target for programming languages. It enables high-performance applications on web pages at near-native speed.

### Key Characteristics:

- **Binary Format**: Compact binary format that's faster to parse than JavaScript
- **Near-Native Speed**: Executes at near-native performance
- **Language Agnostic**: Can be compiled from C, C++, Rust, Go, and more
- **Sandboxed**: Runs in a secure, memory-safe environment
- **Portable**: Works across all modern browsers and platforms

### WASM vs JavaScript:

| Aspect | JavaScript | WebAssembly |
|--------|------------|-------------|
| Parsing | Text-based, slower | Binary, faster |
| Execution | JIT compiled | Pre-compiled |
| Performance | Good | Near-native |
| Memory | Garbage collected | Manual/controlled |
| Use Case | General web | Computation-heavy |

---

## Why WASM for This Project?

### Problem Statement

The Split Bills app needs to calculate bill splits. When users are offline (no internet connection), they cannot reach the server API. We need a way to perform calculations locally.

### Solution: Hybrid Approach

```
┌─────────────────────────────────────────────────────────┐
│                    User Request                         │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│              Hybrid Calculator                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │  1. Check: Is online & server healthy?          │   │
│  └─────────────────────┬───────────────────────────┘   │
│                        │                                │
│         Yes ◄──────────┴──────────► No                 │
│          │                           │                  │
│          ▼                           ▼                  │
│  ┌───────────────┐          ┌───────────────┐          │
│  │  Server API   │          │  WASM Module  │          │
│  │  (Rust/Axum)  │          │  (Rust→WASM)  │          │
│  └───────────────┘          └───────────────┘          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Benefits:

1. **Code Reuse**: Same Rust calculation logic for both server and client
2. **Consistency**: Identical results regardless of which backend is used
3. **Offline Support**: Full functionality without internet
4. **Performance**: WASM calculations are fast
5. **Type Safety**: Rust's type system prevents bugs

---

## Architecture Overview

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
│                                                                  │
│  ┌──────────────┐    ┌─────────────────┐    ┌────────────────┐ │
│  │   script.js  │───▶│ hybrid-         │───▶│ split_bills.js │ │
│  │  (Main App)  │    │ calculator.js   │    │ (WASM Bindings)│ │
│  └──────────────┘    └─────────────────┘    └───────┬────────┘ │
│                              │                       │          │
│                              │                       ▼          │
│                              │              ┌────────────────┐  │
│                              │              │split_bills_bg  │  │
│                              │              │    .wasm       │  │
│                              │              │ (Binary Module)│  │
│                              │              └────────────────┘  │
│                              │                                   │
│                              ▼                                   │
│                     ┌─────────────────┐                         │
│                     │   Server API    │◄────── Network          │
│                     │ /api/calculate  │                         │
│                     └─────────────────┘                         │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### File Structure

```
split-bills/
├── src/
│   └── main.rs              # Server-side calculation (Rust)
├── wasm/
│   ├── Cargo.toml           # WASM crate configuration
│   └── src/
│       └── lib.rs           # WASM calculation logic (Rust)
└── static/
    ├── wasm/
    │   ├── split_bills.js       # Generated JS bindings
    │   └── split_bills_bg.wasm  # Compiled WASM binary
    ├── hybrid-calculator.js     # Hybrid logic (JS)
    ├── script.js                # Main application (JS)
    └── sw.js                    # Service Worker for caching
```

---

## Core Concepts of WASM in Rust

### 1. The `wasm-bindgen` Crate

`wasm-bindgen` is the bridge between Rust and JavaScript. It handles:

- **Type Conversion**: Converting Rust types to JS and vice versa
- **Memory Management**: Passing data between WASM and JS
- **Function Binding**: Exposing Rust functions to JavaScript

```rust
use wasm_bindgen::prelude::*;

// This attribute makes the function callable from JavaScript
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 2. Understanding `#[wasm_bindgen]`

The `#[wasm_bindgen]` attribute is crucial. It tells the compiler how to generate JavaScript bindings.

#### Function Export

```rust
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

This generates JavaScript code that:
1. Converts JS string to WASM memory
2. Calls the Rust function
3. Converts result back to JS string

#### Struct Export

```rust
#[wasm_bindgen]
pub struct Person {
    name: String,
    age: u32,
}

#[wasm_bindgen]
impl Person {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, age: u32) -> Person {
        Person { name, age }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
```

### 3. The `#[wasm_bindgen(start)]` Function

This special attribute marks a function to run when the WASM module is initialized:

```rust
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
}
```

### 4. Memory Model

WASM has a linear memory model - a single contiguous array of bytes:

```
┌─────────────────────────────────────────────────────────┐
│                    WASM Linear Memory                    │
├─────────┬──────────┬──────────┬──────────┬─────────────┤
│  Stack  │   Heap   │  Static  │  Strings │    Free     │
│         │          │   Data   │          │             │
└─────────┴──────────┴──────────┴──────────┴─────────────┘
     ▲                              ▲
     │                              │
     │         JavaScript           │
     │    ┌─────────────────────┐  │
     └────│ getUint8ArrayMemory │──┘
          │ getStringFromWasm   │
          └─────────────────────┘
```

#### String Passing Example

When passing a string from JS to WASM:

1. **JS side**: Encode string to UTF-8 bytes
2. **Allocate**: Request memory in WASM heap
3. **Copy**: Copy bytes to WASM memory
4. **Pass**: Pass pointer and length to Rust function
5. **Rust side**: Reconstruct string from pointer and length

```rust
// This is what wasm-bindgen generates internally
fn pass_string_to_wasm(s: &str) -> (*const u8, usize) {
    let bytes = s.as_bytes();
    let ptr = allocate(bytes.len());
    // Copy bytes to WASM memory at ptr
    (ptr, bytes.len())
}
```

### 5. JsValue and Type Conversion

`JsValue` represents any JavaScript value in Rust:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process(input: JsValue) -> Result<JsValue, JsValue> {
    // Convert JsValue to Rust type
    let data: MyStruct = serde_wasm_bindgen::from_value(input)?;
    
    // Process...
    
    // Convert back to JsValue
    Ok(serde_wasm_bindgen::to_value(&result)?)
}
```

### 6. Error Handling

WASM functions can return `Result<T, JsValue>` for error handling:

```rust
#[wasm_bindgen]
pub fn divide(a: f64, b: f64) -> Result<f64, JsValue> {
    if b == 0.0 {
        Err(JsValue::from_str("Division by zero"))
    } else {
        Ok(a / b)
    }
}
```

In JavaScript:
```javascript
try {
    const result = wasm.divide(10, 0);
} catch (error) {
    console.error(error); // "Division by zero"
}
```

---

## Project Structure

### WASM Crate Configuration (`wasm/Cargo.toml`)

```toml
[package]
name = "split-bills-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]
#            ^^^^^^^^  ^^^^
#            │         └── Regular Rust library (for testing)
#            └── C-compatible dynamic library (for WASM)

[features]
default = ["console_error_panic_hook"]

[dependencies]
wasm-bindgen = "0.2"          # Core WASM bindings
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_json = "1.0"            # JSON support
js-sys = "0.3"                # JavaScript standard library bindings
console_error_panic_hook = { version = "0.1", optional = true }

[profile.release]
opt-level = "s"   # Optimize for size
lto = true        # Link-time optimization
```

### Key Points:

- **`crate-type = ["cdylib"]`**: Required for WASM output
- **`opt-level = "s"`**: Optimizes for smaller file size
- **`lto = true`**: Enables link-time optimization for smaller output

---

## Implementation Details

### The Calculation Logic (`wasm/src/lib.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Data models must match server-side exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub amount_spent: f64,
    // ... other fields
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub people: Vec<Person>,
    pub include_sponsor: bool,
    pub fund_amount: f64,
    pub tip_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CalculateResponse {
    pub total_spent: f64,
    pub settlements: Vec<Settlement>,
    // ... other fields
}

// Main WASM entry point
#[wasm_bindgen]
pub fn calculate_split(request_json: &str) -> Result<String, JsValue> {
    // 1. Parse JSON input
    let request: CalculateRequest = serde_json::from_str(request_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    // 2. Perform calculation (same logic as server)
    let response = calculate_split_internal(request);
    
    // 3. Serialize result to JSON
    serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

// Internal calculation (pure Rust, no WASM specifics)
fn calculate_split_internal(request: CalculateRequest) -> CalculateResponse {
    // ... calculation logic identical to server
}
```

### Why JSON for Data Exchange?

We use JSON strings instead of complex object passing because:

1. **Simplicity**: Easy to debug (human-readable)
2. **Compatibility**: Same format as server API
3. **Flexibility**: Schema changes don't break bindings
4. **Performance**: Serialization overhead is minimal for our data size

```rust
// Instead of this (complex binding):
#[wasm_bindgen]
pub fn calculate(request: CalculateRequest) -> CalculateResponse { }

// We do this (simple string passing):
#[wasm_bindgen]
pub fn calculate_split(request_json: &str) -> Result<String, JsValue> { }
```

---

## Building WASM

### Prerequisites

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build Command

```bash
cd wasm
wasm-pack build --target web --out-dir ../static/wasm --out-name split_bills
```

### Build Options Explained

| Option | Description |
|--------|-------------|
| `--target web` | Generate ES modules for browser use |
| `--target bundler` | Generate for bundlers (webpack, etc.) |
| `--target nodejs` | Generate for Node.js |
| `--out-dir` | Output directory |
| `--out-name` | Base name for output files |

### Output Files

```
static/wasm/
├── split_bills.js           # JavaScript glue code
├── split_bills.d.ts         # TypeScript definitions
├── split_bills_bg.wasm      # Compiled WASM binary
├── split_bills_bg.wasm.d.ts # WASM TypeScript definitions
└── package.json             # NPM package info
```

---

## JavaScript Integration

### Loading WASM Module

```javascript
// hybrid-calculator.js

async function loadWasm() {
    // Dynamic import of WASM bindings
    const wasm = await import('/static/wasm/split_bills.js');
    
    // Initialize WASM module (loads .wasm file)
    await wasm.default();
    
    // Verify it's working
    if (wasm.health_check()) {
        console.log('WASM loaded, version:', wasm.get_version());
        return wasm;
    }
    
    throw new Error('WASM health check failed');
}
```

### Using WASM Functions

```javascript
async function calculateWithWasm(request) {
    const wasm = await loadWasm();
    
    // Convert request to JSON string
    const requestJson = JSON.stringify(request);
    
    // Call WASM function
    const resultJson = wasm.calculate_split(requestJson);
    
    // Parse result
    return JSON.parse(resultJson);
}
```

### Hybrid Calculator Flow

```javascript
export async function calculateSplit(request) {
    // Try server first if online
    if (navigator.onLine && serverHealthy) {
        try {
            return await calculateWithServer(request);
        } catch (error) {
            console.warn('Server failed, falling back to WASM');
        }
    }
    
    // Fall back to WASM
    return await calculateWithWasm(request);
}
```

---

## Memory Management

### Rust Memory in WASM

Unlike JavaScript, WASM/Rust requires explicit memory management. However, `wasm-bindgen` handles most of this automatically.

### String Memory Flow

```
JavaScript                    WASM Linear Memory              Rust
──────────                    ──────────────────              ────
"Hello"          ──Copy──▶    [H,e,l,l,o]        ──Read──▶   String
                              ▲
                              │ ptr, len
                              │
                 ◀──Free──    [freed]            ◀──Drop──   dropped
```

### Important: Memory Leaks

Be careful with complex types. If you return a Rust struct to JS without proper handling, it may leak memory:

```rust
// ❌ Potential memory leak
#[wasm_bindgen]
pub fn create_person() -> Person {
    Person { name: "Alice".to_string() }
}

// ✅ Better: Return serialized data
#[wasm_bindgen]
pub fn create_person_json() -> String {
    let person = Person { name: "Alice".to_string() };
    serde_json::to_string(&person).unwrap()
}
```

---

## Best Practices

### 1. Keep WASM Module Small

```toml
# Cargo.toml
[profile.release]
opt-level = "s"      # Optimize for size (or "z" for even smaller)
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower compile
panic = "abort"      # Smaller binary, no unwinding
```

### 2. Use Feature Flags

```toml
[features]
default = ["console_error_panic_hook"]
debug = ["console_error_panic_hook"]  # Only in debug builds
```

### 3. Minimize Data Crossing

Each JS↔WASM boundary crossing has overhead. Batch operations when possible:

```rust
// ❌ Multiple crossings
for item in items {
    wasm.process_item(item);
}

// ✅ Single crossing
wasm.process_items(JSON.stringify(items));
```

### 4. Handle Errors Gracefully

```rust
#[wasm_bindgen]
pub fn safe_calculate(input: &str) -> Result<String, JsValue> {
    // Validate input
    if input.is_empty() {
        return Err(JsValue::from_str("Input cannot be empty"));
    }
    
    // Parse with error handling
    let request: Request = serde_json::from_str(input)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;
    
    // Process
    let result = process(request);
    
    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}
```

### 5. Test Both Environments

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculation_native() {
        // Test in native Rust
        let request = CalculateRequest { /* ... */ };
        let result = calculate_split_internal(request);
        assert_eq!(result.total_spent, 100.0);
    }
}

// In browser, test via JavaScript
```

---

## Troubleshooting

### Common Issues

#### 1. "wasm-pack not found"

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or via cargo
cargo install wasm-pack
```

#### 2. "Missing wasm32 target"

```bash
rustup target add wasm32-unknown-unknown
```

#### 3. "Module not found" in browser

Check that:
- WASM files are served with correct MIME type (`application/wasm`)
- Paths are correct (absolute vs relative)
- Server allows CORS if needed

#### 4. "RuntimeError: unreachable executed"

Usually a panic in Rust. Enable panic hook for better messages:

```rust
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
```

#### 5. Large WASM file size

```bash
# Check what's taking space
cargo install twiggy
twiggy top target/wasm32-unknown-unknown/release/split_bills_wasm.wasm

# Optimize
wasm-opt -Os -o optimized.wasm original.wasm
```

### Debugging Tips

1. **Use browser DevTools**: Network tab shows WASM loading
2. **Console logging**: Add `web-sys` for `console.log` in Rust
3. **Source maps**: Enable for development builds
4. **Chrome WASM debugging**: chrome://inspect → WebAssembly

---

## Summary

### What We Learned

1. **WASM Basics**: Binary format for high-performance web apps
2. **Rust → WASM**: Use `wasm-bindgen` for seamless integration
3. **Memory Model**: Linear memory with explicit management
4. **Best Practices**: Keep modules small, minimize boundary crossings
5. **Hybrid Approach**: Server for online, WASM for offline

### Key Files in This Project

| File | Purpose |
|------|---------|
| `wasm/Cargo.toml` | WASM crate configuration |
| `wasm/src/lib.rs` | Rust calculation logic for WASM |
| `static/wasm/*.js` | Generated JavaScript bindings |
| `static/wasm/*.wasm` | Compiled WASM binary |
| `static/hybrid-calculator.js` | Hybrid server/WASM logic |
| `static/sw.js` | Service worker for caching |
| `build-wasm.sh` | Build script |

### Commands Reference

```bash
# Build WASM
./build-wasm.sh

# Or manually
cd wasm
wasm-pack build --target web --out-dir ../static/wasm --out-name split_bills

# Run tests
cd wasm
cargo test

# Check WASM size
ls -lh static/wasm/*.wasm
```

---

## Further Reading

- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [wasm-pack Documentation](https://rustwasm.github.io/docs/wasm-pack/)
- [WebAssembly Specification](https://webassembly.github.io/spec/core/)
- [MDN WebAssembly Guide](https://developer.mozilla.org/en-US/docs/WebAssembly)
