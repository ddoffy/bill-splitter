/**
 * Service Worker for Split Bills App
 * 
 * Caches WASM module and essential assets for offline calculation support.
 */

const CACHE_NAME = 'split-bills-v1';
const WASM_CACHE = 'split-bills-wasm-v1';

// Assets to cache for offline use
const STATIC_ASSETS = [
    '/',
    '/static/styles.css',
    '/static/script.js',
    '/static/hybrid-calculator.js',
];

// WASM assets (cached separately for easier updates)
const WASM_ASSETS = [
    '/static/wasm/split_bills.js',
    '/static/wasm/split_bills_bg.wasm',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
    console.log('[SW] Installing service worker...');
    
    event.waitUntil(
        Promise.all([
            // Cache static assets
            caches.open(CACHE_NAME).then((cache) => {
                console.log('[SW] Caching static assets');
                return cache.addAll(STATIC_ASSETS);
            }),
            // Cache WASM assets
            caches.open(WASM_CACHE).then((cache) => {
                console.log('[SW] Caching WASM assets');
                return cache.addAll(WASM_ASSETS).catch((error) => {
                    console.warn('[SW] WASM assets not available yet:', error.message);
                    // Don't fail installation if WASM isn't built yet
                });
            }),
        ]).then(() => {
            console.log('[SW] Installation complete');
            return self.skipWaiting();
        })
    );
});

// Activate event - cleanup old caches
self.addEventListener('activate', (event) => {
    console.log('[SW] Activating service worker...');
    
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames
                    .filter((name) => name !== CACHE_NAME && name !== WASM_CACHE)
                    .map((name) => {
                        console.log('[SW] Deleting old cache:', name);
                        return caches.delete(name);
                    })
            );
        }).then(() => {
            console.log('[SW] Activation complete');
            return self.clients.claim();
        })
    );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    // Handle WASM files - cache first, network fallback
    if (url.pathname.includes('/wasm/')) {
        event.respondWith(
            caches.match(event.request).then((cachedResponse) => {
                if (cachedResponse) {
                    console.log('[SW] Serving WASM from cache:', url.pathname);
                    return cachedResponse;
                }
                
                return fetch(event.request).then((networkResponse) => {
                    // Cache the new WASM file
                    if (networkResponse.ok) {
                        const responseToCache = networkResponse.clone();
                        caches.open(WASM_CACHE).then((cache) => {
                            cache.put(event.request, responseToCache);
                        });
                    }
                    return networkResponse;
                });
            })
        );
        return;
    }
    
    // Handle API requests - network only (with offline fallback for calculate)
    if (url.pathname.startsWith('/api/')) {
        // For calculate API, don't use service worker - let hybrid-calculator.js handle it
        // This allows the WASM fallback to work properly
        event.respondWith(
            fetch(event.request).catch((error) => {
                console.log('[SW] API request failed:', url.pathname, error.message);
                
                // For non-calculate APIs, return an error response
                if (!url.pathname.includes('/api/calculate')) {
                    return new Response(
                        JSON.stringify({ error: 'Offline', message: 'This feature requires an internet connection' }),
                        { status: 503, headers: { 'Content-Type': 'application/json' } }
                    );
                }
                
                // For calculate API, throw to let hybrid-calculator handle with WASM
                throw error;
            })
        );
        return;
    }
    
    // Handle static assets - stale-while-revalidate
    if (url.pathname.startsWith('/static/') || url.pathname === '/') {
        event.respondWith(
            caches.match(event.request).then((cachedResponse) => {
                const networkFetch = fetch(event.request).then((networkResponse) => {
                    // Update cache with fresh version
                    if (networkResponse.ok) {
                        const responseToCache = networkResponse.clone();
                        caches.open(CACHE_NAME).then((cache) => {
                            cache.put(event.request, responseToCache);
                        });
                    }
                    return networkResponse;
                }).catch(() => {
                    // Network failed, return cached version
                    return cachedResponse;
                });
                
                // Return cached version immediately, update in background
                return cachedResponse || networkFetch;
            })
        );
        return;
    }
    
    // Default: network first, cache fallback
    event.respondWith(
        fetch(event.request).catch(() => {
            return caches.match(event.request);
        })
    );
});

// Handle messages from the main thread
self.addEventListener('message', (event) => {
    if (event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }
    
    if (event.data.type === 'CACHE_WASM') {
        // Manually trigger WASM caching
        caches.open(WASM_CACHE).then((cache) => {
            return cache.addAll(WASM_ASSETS);
        }).then(() => {
            event.ports[0].postMessage({ success: true });
        }).catch((error) => {
            event.ports[0].postMessage({ success: false, error: error.message });
        });
    }
    
    if (event.data.type === 'GET_CACHE_STATUS') {
        Promise.all([
            caches.has(CACHE_NAME),
            caches.has(WASM_CACHE),
            caches.open(WASM_CACHE).then((cache) => cache.keys()),
        ]).then(([hasStatic, hasWasm, wasmKeys]) => {
            event.ports[0].postMessage({
                staticCached: hasStatic,
                wasmCached: hasWasm,
                wasmFiles: wasmKeys.map((r) => r.url),
            });
        });
    }
});

console.log('[SW] Service worker loaded');
