/**
 * Hybrid Calculator Service
 * 
 * Uses server API when online, falls back to WASM for offline calculation.
 * This ensures the app works even without network connectivity.
 */

// WASM module state
let wasmModule = null;
let wasmLoading = false;
let wasmLoadPromise = null;
let wasmLoadFailed = false;
let wasmLoadError = null;

// Connection state
let isOnline = navigator.onLine;
let serverHealthy = true;
let lastServerCheck = 0;
const SERVER_CHECK_INTERVAL = 30000; // 30 seconds

/**
 * Initialize the hybrid calculator
 */
export async function initHybridCalculator() {
    // Listen for online/offline events
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    
    // Preload WASM module in background
    preloadWasm();
    
    // Check server health
    checkServerHealth();
    
    console.log('[HybridCalc] Initialized, online:', isOnline);
}

/**
 * Handle coming online
 */
function handleOnline() {
    isOnline = true;
    console.log('[HybridCalc] Online');
    checkServerHealth();
}

/**
 * Handle going offline
 */
function handleOffline() {
    isOnline = false;
    serverHealthy = false;
    console.log('[HybridCalc] Offline - using WASM');
}

/**
 * Check if server is healthy
 */
async function checkServerHealth() {
    if (!isOnline) {
        serverHealthy = false;
        return false;
    }
    
    const now = Date.now();
    if (now - lastServerCheck < SERVER_CHECK_INTERVAL && serverHealthy) {
        return serverHealthy;
    }
    
    try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000);
        
        const response = await fetch('/api/calculate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                people: [{ id: 0, name: 'test', description: '', amount_spent: 0, quantity: 1, tip: 0, is_sponsor: false, sponsor_amount: 0 }],
                include_sponsor: true,
                fund_amount: 0,
                tip_percentage: 0
            }),
            signal: controller.signal
        });
        
        clearTimeout(timeoutId);
        serverHealthy = response.ok;
        lastServerCheck = now;
        
        console.log('[HybridCalc] Server health check:', serverHealthy ? 'OK' : 'FAILED');
        return serverHealthy;
    } catch (error) {
        serverHealthy = false;
        console.log('[HybridCalc] Server unreachable:', error.message);
        return false;
    }
}

/**
 * Preload WASM module in background
 */
async function preloadWasm() {
    if (wasmModule || wasmLoading) return;
    
    wasmLoading = true;
    
    try {
        wasmLoadPromise = loadWasm();
        await wasmLoadPromise;
        console.log('[HybridCalc] WASM preloaded successfully');
    } catch (error) {
        console.warn('[HybridCalc] WASM preload failed:', error.message);
    } finally {
        wasmLoading = false;
    }
}

/**
 * Load WASM module
 */
async function loadWasm() {
    if (wasmModule) return wasmModule;
    if (wasmLoadFailed) throw new Error(wasmLoadError || 'WASM previously failed to load');
    
    try {
        // Dynamic import of WASM bindings
        const wasm = await import('/static/wasm/split_bills.js');
        
        // Initialize WASM - this is where externref issues occur
        try {
            await wasm.default();
        } catch (initError) {
            // Check for externref/Table.grow error
            if (initError.message && initError.message.includes('Table.grow')) {
                console.error('[HybridCalc] WASM externref issue - module incompatible with browser');
                wasmLoadFailed = true;
                wasmLoadError = 'WASM module requires newer browser features. Please rebuild with reference-types disabled.';
                throw new Error(wasmLoadError);
            }
            throw initError;
        }
        
        // Verify it's working
        if (wasm.health_check && wasm.health_check()) {
            wasmModule = wasm;
            console.log('[HybridCalc] WASM module loaded, version:', wasm.get_version());
            return wasmModule;
        } else {
            throw new Error('WASM health check failed');
        }
    } catch (error) {
        console.error('[HybridCalc] Failed to load WASM:', error);
        wasmLoadFailed = true;
        wasmLoadError = error.message;
        throw error;
    }
}

/**
 * Calculate split using server API
 */
async function calculateWithServer(request) {
    const response = await fetch('/api/calculate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request)
    });
    
    if (!response.ok) {
        throw new Error(`Server error: ${response.status}`);
    }
    
    return await response.json();
}

/**
 * Calculate split using WASM
 */
async function calculateWithWasm(request) {
    // Ensure WASM is loaded
    if (!wasmModule) {
        if (wasmLoadPromise) {
            await wasmLoadPromise;
        } else {
            await loadWasm();
        }
    }
    
    if (!wasmModule) {
        throw new Error('WASM module not available');
    }
    
    const resultJson = wasmModule.calculate_split(JSON.stringify(request));
    return JSON.parse(resultJson);
}

/**
 * Main calculation function - tries server first, falls back to WASM
 * 
 * @param {Object} request - The calculation request
 * @param {Array} request.people - List of people
 * @param {boolean} request.include_sponsor - Include sponsors in split
 * @param {number} request.fund_amount - Fund amount
 * @param {number} request.tip_percentage - Tip percentage
 * @returns {Object} Calculation result
 */
export async function calculateSplit(request) {
    const startTime = performance.now();
    let source = 'unknown';
    
    try {
        // If online and server seems healthy, try server first
        if (isOnline && serverHealthy) {
            try {
                const result = await calculateWithServer(request);
                source = 'server';
                logCalculation(startTime, source, true);
                return { ...result, _source: 'server' };
            } catch (serverError) {
                console.warn('[HybridCalc] Server calculation failed, trying WASM:', serverError.message);
                serverHealthy = false;
            }
        }
        
        // Fall back to WASM
        const result = await calculateWithWasm(request);
        source = 'wasm';
        logCalculation(startTime, source, true);
        return { ...result, _source: 'wasm' };
        
    } catch (error) {
        logCalculation(startTime, source, false, error.message);
        throw error;
    }
}

/**
 * Force calculation with specific backend
 */
export async function calculateWithBackend(request, backend = 'auto') {
    if (backend === 'server') {
        return await calculateWithServer(request);
    } else if (backend === 'wasm') {
        return await calculateWithWasm(request);
    } else {
        return await calculateSplit(request);
    }
}

/**
 * Get current calculator status
 */
export function getCalculatorStatus() {
    return {
        isOnline,
        serverHealthy,
        wasmLoaded: !!wasmModule,
        wasmLoading,
        wasmLoadFailed,
        wasmLoadError,
        wasmVersion: wasmModule ? wasmModule.get_version() : null,
        preferredBackend: (isOnline && serverHealthy) ? 'server' : (wasmModule ? 'wasm' : 'none')
    };
}

/**
 * Log calculation for debugging
 */
function logCalculation(startTime, source, success, error = null) {
    const duration = Math.round(performance.now() - startTime);
    
    if (success) {
        console.log(`[HybridCalc] Calculated via ${source} in ${duration}ms`);
    } else {
        console.error(`[HybridCalc] Calculation failed (${source}) in ${duration}ms:`, error);
    }
}

/**
 * Show offline indicator in UI
 */
export function updateOfflineIndicator() {
    const indicator = document.getElementById('offline-indicator');
    if (!indicator) return;
    
    const status = getCalculatorStatus();
    
    if (status.wasmLoadFailed && !status.serverHealthy) {
        indicator.textContent = '❌ Offline Mode Unavailable';
        indicator.className = 'offline-indicator error';
        indicator.style.display = 'block';
    } else if (!status.isOnline) {
        indicator.textContent = '📴 Offline Mode (WASM)';
        indicator.className = 'offline-indicator offline';
        indicator.style.display = 'block';
    } else if (!status.serverHealthy) {
        indicator.textContent = status.wasmLoaded ? '⚠️ Server Unavailable (WASM)' : '⚠️ Connecting...';
        indicator.className = 'offline-indicator degraded';
        indicator.style.display = 'block';
    } else {
        indicator.style.display = 'none';
    }
}

// Export for global access
window.HybridCalculator = {
    init: initHybridCalculator,
    calculate: calculateSplit,
    calculateWithBackend,
    getStatus: getCalculatorStatus,
    updateIndicator: updateOfflineIndicator,
    preloadWasm
};
