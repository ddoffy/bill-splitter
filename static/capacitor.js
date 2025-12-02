// Capacitor initialization
import { Capacitor } from '@capacitor/core';

// Export for use in app
window.Capacitor = Capacitor;
window.isNativeApp = Capacitor.isNativePlatform();

console.log('Running on:', Capacitor.getPlatform());
console.log('Is native app:', window.isNativeApp);
