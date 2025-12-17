// Theme Management
function initializeTheme() {
    const savedTheme = localStorage.getItem('theme') || 'dark';
    applyTheme(savedTheme);
    updateThemeIcon(savedTheme);
}

function applyTheme(theme) {
    if (theme === 'light') {
        document.documentElement.classList.add('light-mode');
    } else {
        document.documentElement.classList.remove('light-mode');
    }
    localStorage.setItem('theme', theme);
}

function toggleTheme() {
    const currentTheme = document.documentElement.classList.contains('light-mode') ? 'light' : 'dark';
    const newTheme = currentTheme === 'light' ? 'dark' : 'light';
    applyTheme(newTheme);
    updateThemeIcon(newTheme);
}

function updateThemeIcon(theme) {
    const themeIcon = document.querySelector('.theme-icon');
    if (themeIcon) {
        if (theme === 'light') {
            themeIcon.classList.remove('mdi-weather-night');
            themeIcon.classList.add('mdi-white-balance-sunny');
        } else {
            themeIcon.classList.remove('mdi-white-balance-sunny');
            themeIcon.classList.add('mdi-weather-night');
        }
    }
}

// PWA Management
let deferredPrompt;
let swRegistration = null;

// Register Service Worker
async function registerServiceWorker() {
    if (!('serviceWorker' in navigator)) {
        console.log('[PWA] Service Workers not supported');
        return;
    }

    try {
        swRegistration = await navigator.serviceWorker.register('/static/sw.js', {
            scope: '/'
        });
        
        console.log('[PWA] Service Worker registered successfully:', swRegistration.scope);
        
        // Check for updates every hour
        setInterval(() => {
            swRegistration.update();
        }, 60 * 60 * 1000);
        
        // Listen for updates
        swRegistration.addEventListener('updatefound', () => {
            const newWorker = swRegistration.installing;
            console.log('[PWA] New service worker installing...');
            
            newWorker.addEventListener('statechange', () => {
                if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                    // New version available
                    console.log('[PWA] New version available!');
                    showUpdateNotification();
                }
            });
        });
        
    } catch (error) {
        console.error('[PWA] Service Worker registration failed:', error);
    }
}

// Show update notification
function showUpdateNotification() {
    const updateBanner = document.createElement('div');
    updateBanner.className = 'pwa-update-banner';
    updateBanner.innerHTML = `
        <div class="pwa-update-content">
            <span class="mdi mdi-refresh-circle"></span>
            <span>A new version is available!</span>
        </div>
        <button class="pwa-update-button" onclick="updateApp()">Update Now</button>
        <button class="pwa-dismiss-button" onclick="dismissUpdate(this)">×</button>
    `;
    document.body.appendChild(updateBanner);
    
    setTimeout(() => {
        updateBanner.classList.add('show');
    }, 100);
}

// Update the app
function updateApp() {
    if (swRegistration && swRegistration.waiting) {
        swRegistration.waiting.postMessage({ type: 'SKIP_WAITING' });
    }
    window.location.reload();
}

// Dismiss update notification
function dismissUpdate(button) {
    const banner = button.closest('.pwa-update-banner');
    banner.classList.remove('show');
    setTimeout(() => {
        banner.remove();
    }, 300);
}

// Handle install prompt
function setupInstallPrompt() {
    window.addEventListener('beforeinstallprompt', (e) => {
        console.log('[PWA] beforeinstallprompt event fired');
        e.preventDefault();
        deferredPrompt = e;
        showInstallButton();
    });
    
    window.addEventListener('appinstalled', () => {
        console.log('[PWA] App installed successfully');
        hideInstallButton();
        showToast('Humidor installed successfully!', 'success');
    });
}

// Show install button
function showInstallButton() {
    const installButton = document.createElement('button');
    installButton.id = 'pwaInstallButton';
    installButton.className = 'pwa-install-button';
    installButton.innerHTML = `
        <span class="mdi mdi-download"></span>
        <span>Install App</span>
    `;
    installButton.addEventListener('click', installApp);
    
    // Add to header-right section
    const headerRight = document.querySelector('.header-right');
    if (headerRight) {
        headerRight.insertBefore(installButton, headerRight.firstChild);
    }
}

// Hide install button
function hideInstallButton() {
    const installButton = document.getElementById('pwaInstallButton');
    if (installButton) {
        installButton.remove();
    }
}

// Install the app
async function installApp() {
    if (!deferredPrompt) {
        console.log('[PWA] No deferred prompt available');
        return;
    }
    
    deferredPrompt.prompt();
    
    const { outcome } = await deferredPrompt.userChoice;
    console.log('[PWA] User choice:', outcome);
    
    if (outcome === 'accepted') {
        console.log('[PWA] User accepted the install prompt');
    } else {
        console.log('[PWA] User dismissed the install prompt');
    }
    
    deferredPrompt = null;
    hideInstallButton();
}

// Mobile Menu Management
function initializeMobileMenu() {
    const mobileMenuToggle = document.getElementById('mobileMenuToggle');
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!mobileMenuToggle || !sidebar || !backdrop) {
        console.log('Mobile menu elements not found');
        return;
    }
    
    // Toggle sidebar on button click
    mobileMenuToggle.addEventListener('click', () => {
        toggleMobileMenu();
    });
    
    // Close sidebar on backdrop click
    backdrop.addEventListener('click', (e) => {
        // Always close the mobile menu when backdrop is clicked
        if (sidebar.classList.contains('open')) {
            closeMobileMenu();
        }
    });
    
    // Close sidebar when navigation item is clicked
    const navItems = sidebar.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.addEventListener('click', () => {
            // Small delay to allow navigation to complete
            setTimeout(() => {
                closeMobileMenu();
            }, 100);
        });
    });
    
    // Close sidebar on window resize if going above mobile breakpoint
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            if (window.innerWidth > 1024) {
                closeMobileMenu();
            }
        }, 250);
    });
}

function toggleMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    const isOpen = sidebar.classList.contains('open');
    
    if (isOpen) {
        closeMobileMenu();
    } else {
        openMobileMenu();
    }
}

function openMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    sidebar.classList.add('open');
    backdrop.classList.add('show');
    
    // Prevent body scroll when menu is open
    document.body.style.overflow = 'hidden';
}

function closeMobileMenu() {
    const sidebar = document.getElementById('sidebar');
    const backdrop = document.getElementById('sidebarBackdrop');
    
    if (!sidebar || !backdrop) return;
    
    sidebar.classList.remove('open');
    backdrop.classList.remove('show');
    
    // Restore body scroll
    document.body.style.overflow = '';
}

// Mobile Filter Toggle Management
function initializeMobileFilterToggle() {
    const filterToggle = document.getElementById('mobileFilterToggle');
    const filtersWrapper = document.getElementById('searchFiltersWrapper');
    
    if (!filterToggle || !filtersWrapper) {
        return;
    }
    
    // Toggle filters on button click
    filterToggle.addEventListener('click', () => {
        const isExpanded = filtersWrapper.classList.contains('mobile-expanded');
        
        if (isExpanded) {
            filtersWrapper.classList.remove('mobile-expanded');
            filtersWrapper.classList.add('mobile-collapsed');
            filterToggle.classList.remove('active');
        } else {
            filtersWrapper.classList.remove('mobile-collapsed');
            filtersWrapper.classList.add('mobile-expanded');
            filterToggle.classList.add('active');
        }
    });
    
    // Auto-collapse filters on window resize if going above mobile breakpoint
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            if (window.innerWidth > 768) {
                // On desktop, show filters by default (remove mobile classes)
                filtersWrapper.classList.remove('mobile-collapsed', 'mobile-expanded');
                filterToggle.classList.remove('active');
            } else {
                // On mobile, ensure collapsed state if not already set
                if (!filtersWrapper.classList.contains('mobile-expanded')) {
                    filtersWrapper.classList.add('mobile-collapsed');
                }
            }
        }, 250);
    });
    
    // Update filter toggle badge when filters change
    updateFilterToggleBadge();
}

function updateFilterToggleBadge() {
    const filterToggleBadge = document.getElementById('filterToggleBadge');
    
    if (!filterToggleBadge) return;
    
    // Count active filters
    const activeFilters = [
        selectedBrands.length,
        selectedSizes.length,
        selectedOrigins.length,
        selectedStrengths.length,
        selectedRingGauges.length
    ].reduce((sum, count) => sum + count, 0);
    
    if (activeFilters > 0) {
        filterToggleBadge.textContent = activeFilters;
        filterToggleBadge.style.display = 'inline-flex';
    } else {
        filterToggleBadge.style.display = 'none';
    }
}

// Application State
let humidors = [];
let cigars = [];
let wishListCigars = [];
let currentHumidor = null;
let currentHumidorPermission = 'full'; // 'view', 'edit', or 'full'
let currentCigar = null;
let isEditingHumidor = false;
let isEditingCigar = false;
let currentPage = 'humidors';
let currentUser = null;
let authToken = null;

// Organizer State
let brands = [];
let sizes = [];
let origins = [];
let strengths = [];
let ringGauges = [];
let currentOrganizer = null;
let isEditingOrganizer = false;

// Import State
let scrapedCigarData = null;

// Search and Filter State
let searchQuery = '';
let selectedBrands = [];
let selectedSizes = [];
let selectedOrigins = [];
let selectedStrengths = [];
let selectedRingGauges = [];
let filteredCigars = [];

// Pagination State
let currentCigarPage = 1;
let cigarPageSize = 50;
let totalCigars = 0;
let totalCigarPages = 0;

// Router State
let currentRoute = { view: 'hub', humidorId: null };

// Router Functions
function initializeRouter() {
    // Handle hash changes
    window.addEventListener('hashchange', handleRouteChange);
    
    // Handle initial route
    handleRouteChange();
}

// Theme Toggle Event Listener
function setupThemeToggle() {
    const themeToggle = document.getElementById('themeToggle');
    if (themeToggle) {
        themeToggle.addEventListener('click', toggleTheme);
    }
}

function handleRouteChange() {
    const hash = window.location.hash.slice(1); // Remove the # character
    console.log('Route changed to:', hash);
    
    if (!hash || hash === 'humidors') {
        // Hub view (or default)
        currentRoute = { view: 'hub', humidorId: null };
        if (humidors.length === 1) {
            // If only one humidor, go directly to it
            console.log('Single humidor, showing detail');
            showHumidorDetail(humidors[0].id);
        } else if (humidors.length > 1) {
            // Show hub with multiple humidors
            console.log('Multiple humidors, showing hub');
            showHumidorHub();
        } else {
            // No humidors yet, show existing empty state
            console.log('No humidors, loading');
            loadHumidors();
        }
    } else if (hash.startsWith('humidors/')) {
        // Detail view for specific humidor
        const humidorId = hash.split('/')[1];
        console.log('Detail route, humidor ID:', humidorId);
        if (humidorId) {
            currentRoute = { view: 'detail', humidorId };
            showHumidorDetail(humidorId);
        } else {
            // Invalid ID, go to hub
            console.log('Invalid humidor ID');
            navigateToHub();
        }
    } else {
        // Unknown route, go to hub
        console.log('Unknown route');
        navigateToHub();
    }
}

function navigateToHub() {
    console.log('Navigating to hub');
    window.location.hash = '#humidors';
}

function navigateToHumidorDetail(humidorId) {
    console.log('Navigating to humidor detail:', humidorId);
    window.location.hash = `#humidors/${humidorId}`;
}

// Utility function to escape HTML
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Helper functions to get organizer names from IDs
function getBrandName(brandId) {
    if (!brandId) {
        console.log('getBrandName: brandId is null/undefined');
        return 'Unknown Brand';
    }
    const brand = brands.find(b => b.id === brandId);
    if (!brand) {
        console.warn(`getBrandName: No brand found for ID ${brandId}. Available brands:`, brands.length);
        return 'Unknown Brand';
    }
    return brand.name;
}

function getSizeName(sizeId) {
    if (!sizeId) {
        console.log('getSizeName: sizeId is null/undefined');
        return 'Unknown Size';
    }
    const size = sizes.find(s => s.id === sizeId);
    if (!size) {
        console.warn(`getSizeName: No size found for ID ${sizeId}. Available sizes:`, sizes.length);
        return 'Unknown Size';
    }
    return size.name;
}

function getOriginName(originId) {
    if (!originId) {
        console.log('getOriginName: originId is null/undefined');
        return 'Unknown Origin';
    }
    const origin = origins.find(o => o.id === originId);
    if (!origin) {
        console.warn(`getOriginName: No origin found for ID ${originId}. Available origins:`, origins.length);
        return 'Unknown Origin';
    }
    return origin.name;
}

function getStrengthName(strengthId) {
    if (!strengthId) {
        console.log('getStrengthName: strengthId is null/undefined');
        return 'Unknown Strength';
    }
    const strength = strengths.find(s => s.id === strengthId);
    if (!strength) {
        console.warn(`getStrengthName: No strength found for ID ${strengthId}. Available strengths:`, strengths.length);
        return 'Unknown Strength';
    }
    return strength.name;
}

function getStrengthLevel(strengthId) {
    if (!strengthId) {
        return null;
    }
    const strength = strengths.find(s => s.id === strengthId);
    return strength ? strength.level : null;
}

function getRingGaugeName(ringGaugeId) {
    if (!ringGaugeId) {
        console.log('getRingGaugeName: ringGaugeId is null/undefined');
        return 'N/A';
    }
    const ringGauge = ringGauges.find(rg => rg.id === ringGaugeId);
    if (!ringGauge) {
        console.warn(`getRingGaugeName: No ring gauge found for ID ${ringGaugeId}. Available ring gauges:`, ringGauges.length);
        return 'N/A';
    }
    return ringGauge.gauge.toString();
}

// DOM Elements - will be initialized after DOM loads
let elements = {};

// Authentication Functions
function checkAuth() {
    authToken = localStorage.getItem('humidor_token');
    const userStr = localStorage.getItem('humidor_user');
    
    if (!authToken || !userStr) {
        return false;
    }
    
    try {
        currentUser = JSON.parse(userStr);
        return true;
    } catch (error) {
        console.error('Error parsing user data:', error);
        logout();
        return false;
    }
}

function logout() {
    localStorage.removeItem('humidor_token');
    localStorage.removeItem('humidor_user');
    window.location.href = '/login.html';
}

function initializeUserDisplay() {
    if (currentUser) {
        // Update user info in header
        const userInfo = document.getElementById('userInfo');
        const userName = document.getElementById('userName');
        const userAvatar = document.getElementById('userAvatar');
        const userDropdownAvatar = document.getElementById('userDropdownAvatar');
        const userDropdownName = document.getElementById('userDropdownName');
        const userDropdownUsername = document.getElementById('userDropdownUsername');
        
        if (userInfo && userName) {
            const displayName = currentUser.full_name || currentUser.username;
            const initials = getInitials(displayName);
            
            userName.textContent = displayName;
            if (userAvatar) userAvatar.textContent = initials;
            if (userDropdownAvatar) userDropdownAvatar.textContent = initials;
            if (userDropdownName) userDropdownName.textContent = displayName;
            if (userDropdownUsername) userDropdownUsername.textContent = `@${currentUser.username}`;
            
            userInfo.style.display = 'block';
        }
    } else {
        // Hide user info if no user
        const userInfo = document.getElementById('userInfo');
        if (userInfo) {
            userInfo.style.display = 'none';
        }
    }
}

// Helper function to get user initials
function getInitials(name) {
    if (!name) return '?';
    const parts = name.trim().split(/\s+/);
    if (parts.length >= 2) {
        return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }
    return name.substring(0, 2).toUpperCase();
}

// API Utility Functions
async function makeAuthenticatedRequest(url, options = {}) {
    const token = localStorage.getItem('humidor_token');
    if (!token) {
        throw new Error('No authentication token found');
    }

    // Build headers - don't set Content-Type if explicitly null (for FormData uploads)
    const headers = {
        'Authorization': `Bearer ${token}`,
        ...(options.headers || {})
    };
    
    // Only add Content-Type if not explicitly excluded and not FormData
    if (options.headers && options.headers['Content-Type'] === null) {
        // Don't set Content-Type - let browser set it for multipart/form-data
        delete headers['Content-Type'];
    } else if (!options.headers || options.headers['Content-Type'] === undefined) {
        // Default to JSON for regular requests
        headers['Content-Type'] = 'application/json';
    }

    const requestOptions = {
        ...options,
        headers
    };

    const response = await fetch(url, requestOptions);
    
    if (response.status === 401) {
        // Token expired or invalid
        localStorage.removeItem('humidor_token');
        localStorage.removeItem('humidor_user');
        window.location.href = '/login.html';
        throw new Error('Authentication failed');
    }
    
    if (response.status === 413) {
        // Payload too large
        throw new Error('PAYLOAD_TOO_LARGE');
    }
    
    return response;
}

// API Functions
const API = {
    async getCigars(params = {}) {
        // Add pagination parameters if not provided
        if (!params.page) params.page = currentCigarPage;
        if (!params.page_size) params.page_size = cigarPageSize;
        
        const searchParams = new URLSearchParams(params);
        const response = await fetch(`/api/v1/cigars?${searchParams}`);
        if (!response.ok) throw new Error('Failed to fetch cigars');
        return response.json();
    },

    async getCigar(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'GET'
        });
        if (!response.ok) throw new Error('Failed to fetch cigar');
        return response.json();
    },

    async createCigar(cigar) {
        const response = await makeAuthenticatedRequest('/api/v1/cigars', {
            method: 'POST',
            body: JSON.stringify(cigar)
        });
        if (!response.ok) throw new Error('Failed to create cigar');
        return response.json();
    },

    async updateCigar(id, cigar) {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'PUT',
            body: JSON.stringify(cigar)
        });
        if (!response.ok) throw new Error('Failed to update cigar');
        return response.json();
    },

    async deleteCigar(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete cigar');
        return response.json();
    }
};

// Organizer API Functions
const OrganizerAPI = {
    // Brand API
    async getBrands() {
        const response = await makeAuthenticatedRequest('/api/v1/brands');
        if (!response.ok) throw new Error('Failed to fetch brands');
        return response.json();
    },

    async createBrand(brand) {
        const response = await makeAuthenticatedRequest('/api/v1/brands', {
            method: 'POST',
            body: JSON.stringify(brand)
        });
        if (!response.ok) throw new Error('Failed to create brand');
        return response.json();
    },

    async updateBrand(id, brand) {
        const response = await makeAuthenticatedRequest(`/api/v1/brands/${id}`, {
            method: 'PUT',
            body: JSON.stringify(brand)
        });
        if (!response.ok) throw new Error('Failed to update brand');
        return response.json();
    },

    async deleteBrand(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/brands/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete brand');
        return response.json();
    },

    // Size API
    async getSizes() {
        const response = await makeAuthenticatedRequest('/api/v1/sizes');
        if (!response.ok) throw new Error('Failed to fetch sizes');
        return response.json();
    },

    async createSize(size) {
        const response = await makeAuthenticatedRequest('/api/v1/sizes', {
            method: 'POST',
            body: JSON.stringify(size)
        });
        if (!response.ok) throw new Error('Failed to create size');
        return response.json();
    },

    async updateSize(id, size) {
        const response = await makeAuthenticatedRequest(`/api/v1/sizes/${id}`, {
            method: 'PUT',
            body: JSON.stringify(size)
        });
        if (!response.ok) throw new Error('Failed to update size');
        return response.json();
    },

    async deleteSize(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/sizes/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete size');
        return response.json();
    },

    // Origin API
    async getOrigins() {
        const response = await makeAuthenticatedRequest('/api/v1/origins');
        if (!response.ok) throw new Error('Failed to fetch origins');
        return response.json();
    },

    async createOrigin(origin) {
        const response = await makeAuthenticatedRequest('/api/v1/origins', {
            method: 'POST',
            body: JSON.stringify(origin)
        });
        if (!response.ok) throw new Error('Failed to create origin');
        return response.json();
    },

    async updateOrigin(id, origin) {
        const response = await makeAuthenticatedRequest(`/api/v1/origins/${id}`, {
            method: 'PUT',
            body: JSON.stringify(origin)
        });
        if (!response.ok) throw new Error('Failed to update origin');
        return response.json();
    },

    async deleteOrigin(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/origins/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete origin');
        return response.json();
    },

    // Strength API
    async getStrengths() {
        const response = await makeAuthenticatedRequest('/api/v1/strengths');
        if (!response.ok) throw new Error('Failed to fetch strengths');
        return response.json();
    },

    async createStrength(strength) {
        const response = await makeAuthenticatedRequest('/api/v1/strengths', {
            method: 'POST',
            body: JSON.stringify(strength)
        });
        if (!response.ok) throw new Error('Failed to create strength');
        return response.json();
    },

    async updateStrength(id, strength) {
        const response = await makeAuthenticatedRequest(`/api/v1/strengths/${id}`, {
            method: 'PUT',
            body: JSON.stringify(strength)
        });
        if (!response.ok) throw new Error('Failed to update strength');
        return response.json();
    },

    async deleteStrength(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/strengths/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete strength');
        return response.json();
    },

    // Ring Gauge API
    async getRingGauges() {
        const response = await makeAuthenticatedRequest('/api/v1/ring-gauges');
        if (!response.ok) throw new Error('Failed to fetch ring gauges');
        return response.json();
    },

    async createRingGauge(ringGauge) {
        const response = await makeAuthenticatedRequest('/api/v1/ring-gauges', {
            method: 'POST',
            body: JSON.stringify(ringGauge)
        });
        if (!response.ok) throw new Error('Failed to create ring gauge');
        return response.json();
    },

    async updateRingGauge(id, ringGauge) {
        const response = await makeAuthenticatedRequest(`/api/v1/ring-gauges/${id}`, {
            method: 'PUT',
            body: JSON.stringify(ringGauge)
        });
        if (!response.ok) throw new Error('Failed to update ring gauge');
        return response.json();
    },

    async deleteRingGauge(id) {
        const response = await makeAuthenticatedRequest(`/api/v1/ring-gauges/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete ring gauge');
        return response.json();
    }
};

// Favorites API Functions
const FavoritesAPI = {
    async getFavorites() {
        const response = await makeAuthenticatedRequest('/api/v1/favorites', {
            method: 'GET'
        });
        if (!response.ok) throw new Error('Failed to fetch favorites');
        return response.json();
    },

    async addFavorite(cigarId) {
        const response = await makeAuthenticatedRequest('/api/v1/favorites', {
            method: 'POST',
            body: JSON.stringify({ cigar_id: cigarId })
        });
        if (!response.ok) throw new Error('Failed to add favorite');
        return response.json();
    },

    async removeFavorite(cigarId) {
        const response = await makeAuthenticatedRequest(`/api/v1/favorites/${cigarId}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to remove favorite');
        return response.json();
    },

    async isFavorite(cigarId) {
        const response = await makeAuthenticatedRequest(`/api/v1/favorites/${cigarId}/check`, {
            method: 'GET'
        });
        if (!response.ok) throw new Error('Failed to check favorite status');
        return response.json();
    }
};

// Utility Functions
function showToast(message, type = 'success') {
    console.log('[TOAST] Creating toast:', message, 'type:', type);
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.innerHTML = `
        <div class="toast-message">${message}</div>
        <button class="toast-close" style="background: none; border: none; color: inherit; font-size: 1.25rem; cursor: pointer; padding: 0 0.5rem; margin-left: 1rem; opacity: 0.7;">×</button>
    `;
    toast.style.display = 'flex';
    toast.style.alignItems = 'center';
    toast.style.justifyContent = 'space-between';
    
    if (!elements.toastContainer) {
        console.error('[TOAST] toastContainer element not found!');
        return;
    }
    
    elements.toastContainer.appendChild(toast);
    console.log('[TOAST] Toast appended to DOM, will remove in 5 seconds');
    
    // Remove toast after 5 seconds with smooth fade-out
    const timeoutId = setTimeout(() => {
        console.log('[TOAST] 5 seconds elapsed, removing toast:', message);
        toast.style.opacity = '0';
        toast.style.transform = 'translateX(100%)';
        toast.style.transition = 'all 0.3s ease-out';
        setTimeout(() => {
            console.log('[TOAST] Toast removed from DOM');
            toast.remove();
        }, 300);
    }, 5000);
    
    // Allow manual dismissal
    toast.querySelector('.toast-close').addEventListener('click', () => {
        console.log('[TOAST] Manual close clicked');
        clearTimeout(timeoutId);
        toast.style.opacity = '0';
        toast.style.transform = 'translateX(100%)';
        toast.style.transition = 'all 0.3s ease-out';
        setTimeout(() => toast.remove(), 300);
    });
}

function formatPrice(price) {
    return price ? `$${parseFloat(price).toFixed(2)}` : 'N/A';
}

function formatDate(dateString) {
    if (!dateString) return 'N/A';
    return new Date(dateString).toLocaleDateString();
}

function getStrengthColor(strength) {
    switch (strength?.toLowerCase()) {
        case 'mild': return '#48bb78';
        case 'medium': return '#ed8936';
        case 'full': return '#e53e3e';
        default: return '#718096';
    }
}

// UI Functions
function createCigarCard(cigar) {
    const card = document.createElement('div');
    card.className = 'cigar-card';
    const isOutOfStock = !cigar.is_active;
    
    // Out of stock badge centered on card
    const outOfStockBadge = isOutOfStock ? `<div class="out-of-stock-badge" style="position: absolute; top: 10px; left: 50%; transform: translateX(-50%); background: #e74c3c; color: white; padding: 4px 12px; border-radius: 4px; font-size: 0.875rem; font-weight: 600; z-index: 10; white-space: nowrap;">OUT OF STOCK</div>` : '';
    
    // Check permission level and conditionally render buttons
    let actionButtons = '';
    if (currentHumidorPermission === 'view') {
        // View only - no action buttons
        actionButtons = '';
    } else if (currentHumidorPermission === 'edit') {
        // Edit permission - can edit and restock but not delete
        actionButtons = isOutOfStock 
            ? `<button class="action-btn edit-btn" onclick="restockCigar('${cigar.id}')" title="Restock" style="background: #27ae60;">↻</button>`
            : `<button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')">✏️</button>`;
    } else {
        // Full permission (default for owners)
        actionButtons = isOutOfStock 
            ? `<button class="action-btn edit-btn" onclick="restockCigar('${cigar.id}')" title="Restock" style="background: #27ae60;">↻</button>`
            : `<button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')">✏️</button>
               <button class="action-btn delete-btn" onclick="deleteCigar('${cigar.id}')">🗑️</button>`;
    }
    
    const quantityDisplay = isOutOfStock ? '<div class="quantity-badge" style="color: #e74c3c;">0 left</div>' : `<div class="quantity-badge">${cigar.quantity} left</div>`;
    
    card.innerHTML = `
        <div class="cigar-card-image" style="position: relative;">
            ${outOfStockBadge}
            <div class="cigar-card-overlay">
                <strong>Notes:</strong> ${cigar.notes || 'No notes available'}
                <br><br>
                <strong>Wrapper:</strong> ${cigar.wrapper || 'N/A'}
                <br>
                <strong>Binder:</strong> ${cigar.binder || 'N/A'}
                <br>
                <strong>Filler:</strong> ${cigar.filler || 'N/A'}
            </div>
        </div>
        <div class="cigar-card-content">
            <div class="cigar-header">
                <div class="cigar-brand">${getBrandName(cigar.brand_id)}</div>
                <div class="cigar-actions">
                    ${actionButtons}
                </div>
            </div>
            
            <div class="cigar-name">${cigar.name}</div>
            
            <div class="cigar-details">
                <div class="detail-item">
                    <div class="detail-label">Size</div>
                    <div class="detail-value">${getSizeName(cigar.size_id)}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Strength</div>
                    <div class="detail-value" style="color: ${getStrengthColor(getStrengthName(cigar.strength_id))}">${getStrengthName(cigar.strength_id)}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Origin</div>
                    <div class="detail-value">${getOriginName(cigar.origin_id)}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Location</div>
                    <div class="detail-value">${cigar.humidor_location || 'Not specified'}</div>
                </div>
            </div>
            
            <div class="cigar-footer">
                ${quantityDisplay}
                <div class="price-tag">${formatPrice(cigar.price)}</div>
            </div>
        </div>
    `;
    
    if (isOutOfStock) {
        card.style.opacity = '0.7';
    }
    
    return card;
}

function updateStats() {
    const totalQuantity = cigars.reduce((sum, cigar) => sum + cigar.quantity, 0);
    const uniqueBrands = new Set(cigars.map(cigar => cigar.brand_id).filter(id => id)).size;
    const totalValue = cigars.reduce((sum, cigar) => {
        const price = parseFloat(cigar.price) || 0;
        return sum + (price * cigar.quantity);
    }, 0);

    elements.totalCigars.textContent = totalQuantity;
    elements.uniqueBrands.textContent = uniqueBrands;
    elements.totalValue.textContent = formatPrice(totalValue);
    
    // Update navigation counts
    updateNavigationCounts();
}

function updateNavigationCounts() {
    const totalQuantity = cigars.reduce((sum, cigar) => sum + cigar.quantity, 0);
    const uniqueBrands = new Set(cigars.map(cigar => cigar.brand)).size;
    const humidorLocations = new Set(cigars.map(cigar => cigar.humidor_location).filter(loc => loc)).size;
    
    // Update counts in navigation
    const allCigarsCount = document.querySelector('[data-page="all-cigars"] .nav-count');
    const brandsCount = document.querySelector('[data-page="brands"] .nav-count');
    const humidorsCount = document.querySelector('[data-page="humidors"] .nav-count');
    
    if (allCigarsCount) allCigarsCount.textContent = totalQuantity;
    if (brandsCount) brandsCount.textContent = uniqueBrands;
    if (humidorsCount) humidorsCount.textContent = humidorLocations;
}

function updateFilters() {
    // Update brand filter using the global brands array
    if (brands && brands.length > 0) {
        elements.brandFilter.innerHTML = '<option value="">All Brands</option>';
        brands.forEach(brand => {
            const option = document.createElement('option');
            option.value = brand.id;
            option.textContent = brand.name;
            elements.brandFilter.appendChild(option);
        });
    }

    // Update origin filter using the global origins array
    if (origins && origins.length > 0) {
        elements.originFilter.innerHTML = '<option value="">All Origins</option>';
        origins.forEach(origin => {
            const option = document.createElement('option');
            option.value = origin.id;
            option.textContent = origin.name;
            elements.originFilter.appendChild(option);
        });
    }

    // Update strength filter using the global strengths array
    if (strengths && strengths.length > 0) {
        elements.strengthFilter.innerHTML = '<option value="">All Strengths</option>';
        strengths.forEach(strength => {
            const option = document.createElement('option');
            option.value = strength.id;
            option.textContent = strength.name;
            elements.strengthFilter.appendChild(option);
        });
    }
}

function renderCigars() {
    elements.cigarsGrid.innerHTML = '';
    
    if (filteredCigars.length === 0) {
        elements.emptyState.style.display = 'block';
        elements.cigarsGrid.style.display = 'none';
    } else {
        elements.emptyState.style.display = 'none';
        elements.cigarsGrid.style.display = 'grid';
        
        filteredCigars.forEach(cigar => {
            const card = createCigarCard(cigar);
            elements.cigarsGrid.appendChild(card);
        });
    }
}

function filterCigars() {
    const searchTerm = elements.searchInput.value.toLowerCase();
    const brandFilter = elements.brandFilter.value ? parseInt(elements.brandFilter.value) : null;
    const strengthFilter = elements.strengthFilter.value ? parseInt(elements.strengthFilter.value) : null;
    const originFilter = elements.originFilter.value ? parseInt(elements.originFilter.value) : null;

    filteredCigars = cigars.filter(cigar => {
        const matchesSearch = !searchTerm || 
            getBrandName(cigar.brand_id).toLowerCase().includes(searchTerm) ||
            cigar.name.toLowerCase().includes(searchTerm) ||
            (cigar.notes && cigar.notes.toLowerCase().includes(searchTerm));
        
        const matchesBrand = !brandFilter || cigar.brand_id === brandFilter;
        const matchesStrength = !strengthFilter || cigar.strength_id === strengthFilter;
        const matchesOrigin = !originFilter || cigar.origin_id === originFilter;

        return matchesSearch && matchesBrand && matchesStrength && matchesOrigin;
    });

    renderCigars();
}

// Pagination Functions
function updatePaginationControls() {
    const paginationContainer = document.getElementById('paginationContainer');
    const paginationInfo = document.getElementById('paginationInfo');
    const paginationPages = document.getElementById('paginationPages');
    const firstPageBtn = document.getElementById('firstPageBtn');
    const prevPageBtn = document.getElementById('prevPageBtn');
    const nextPageBtn = document.getElementById('nextPageBtn');
    const lastPageBtn = document.getElementById('lastPageBtn');
    
    if (!paginationContainer) return;
    
    // Show/hide pagination based on whether we have cigars
    if (totalCigars > 0) {
        paginationContainer.style.display = 'flex';
        
        // Update info text
        const startItem = (currentCigarPage - 1) * cigarPageSize + 1;
        const endItem = Math.min(currentCigarPage * cigarPageSize, totalCigars);
        paginationInfo.textContent = `Showing ${startItem}-${endItem} of ${totalCigars} cigars`;
        
        // Update button states
        firstPageBtn.disabled = currentCigarPage === 1;
        prevPageBtn.disabled = currentCigarPage === 1;
        nextPageBtn.disabled = currentCigarPage === totalCigarPages;
        lastPageBtn.disabled = currentCigarPage === totalCigarPages;
        
        // Generate page numbers
        paginationPages.innerHTML = '';
        const maxVisiblePages = 5;
        let startPage = Math.max(1, currentCigarPage - Math.floor(maxVisiblePages / 2));
        let endPage = Math.min(totalCigarPages, startPage + maxVisiblePages - 1);
        
        // Adjust start if we're near the end
        if (endPage - startPage < maxVisiblePages - 1) {
            startPage = Math.max(1, endPage - maxVisiblePages + 1);
        }
        
        for (let i = startPage; i <= endPage; i++) {
            const pageBtn = document.createElement('button');
            pageBtn.className = 'pagination-page-btn' + (i === currentCigarPage ? ' active' : '');
            pageBtn.textContent = i;
            pageBtn.onclick = () => goToPage(i);
            paginationPages.appendChild(pageBtn);
        }
    } else {
        paginationContainer.style.display = 'none';
    }
}

async function goToPage(page) {
    if (page < 1 || page > totalCigarPages || page === currentCigarPage) return;
    currentCigarPage = page;
    await loadCigars();
}

async function nextPage() {
    if (currentCigarPage < totalCigarPages) {
        await goToPage(currentCigarPage + 1);
    }
}

async function previousPage() {
    if (currentCigarPage > 1) {
        await goToPage(currentCigarPage - 1);
    }
}

async function firstPage() {
    await goToPage(1);
}

async function lastPage() {
    await goToPage(totalCigarPages);
}

async function changePageSize(newSize) {
    cigarPageSize = parseInt(newSize);
    currentCigarPage = 1; // Reset to first page when changing page size
    await loadCigars();
}

function closeCigarModal() {
    elements.cigarModal.classList.remove('show');
    document.body.style.overflow = 'auto';
    currentCigar = null;
    isEditing = false;
}

// Event Handlers
async function handleFormSubmit(event) {
    event.preventDefault();
    
    const formData = new FormData(elements.cigarForm);
    const cigarData = {};
    
    for (const [key, value] of formData.entries()) {
        if (value.trim() !== '') {
            if (key === 'price') {
                cigarData[key] = parseFloat(value);
            } else if (key === 'quantity') {
                cigarData[key] = parseInt(value);
            } else if (key === 'purchase_date') {
                cigarData[key] = new Date(value).toISOString();
            } else {
                cigarData[key] = value;
            }
        }
    }
    
    try {
        elements.saveBtn.disabled = true;
        elements.saveBtn.textContent = isEditing ? 'Updating...' : 'Saving...';
        
        if (isEditing) {
            await API.updateCigar(currentCigar.id, cigarData);
            showToast('Cigar updated successfully!');
        } else {
            await API.createCigar(cigarData);
            showToast('Cigar added successfully!');
        }
        
        closeCigarModal();
        await loadCigars();
    } catch (error) {
        console.error('Error saving cigar:', error);
        showToast('Failed to save cigar', 'error');
    } finally {
        elements.saveBtn.disabled = false;
        elements.saveBtn.textContent = isEditing ? 'Update Cigar' : 'Save Cigar';
    }
}

async function editCigar(id) {
    // Check permission
    if (currentHumidorPermission === 'view') {
        showToast('You only have view permission for this humidor', 'error');
        return;
    }
    
    try {
        const cigar = await API.getCigar(id);
        openCigarModal(cigar.humidor_id, cigar);
    } catch (error) {
        console.error('Error fetching cigar:', error);
        showToast('Failed to load cigar details', 'error');
    }
}

async function deleteCigar(id) {
    // Check permission
    if (currentHumidorPermission !== 'full') {
        showToast('You need full permission to delete cigars', 'error');
        return;
    }
    
    if (!confirm('Are you sure you want to delete this cigar?')) {
        return;
    }
    
    try {
        await API.deleteCigar(id);
        showToast('Cigar deleted successfully!');
        await loadCigars();
    } catch (error) {
        console.error('Error deleting cigar:', error);
        showToast('Failed to delete cigar', 'error');
    }
}

// Main Functions
async function loadCigars() {
    try {
        elements.loading.style.display = 'block';
        elements.cigarsGrid.style.display = 'none';
        elements.emptyState.style.display = 'none';
        
        console.log('Loading cigars...');
        const response = await API.getCigars();
        console.log('Cigars loaded successfully:', response);
        
        // Extract cigars and pagination metadata
        cigars = response.cigars || [];
        totalCigars = response.total || 0;
        currentCigarPage = response.page || 1;
        cigarPageSize = response.page_size || 50;
        totalCigarPages = response.total_pages || 0;
        
        filteredCigars = [...cigars];
        
        updateStats();
        updateFilters();
        renderCigars();
        updatePaginationControls();
    } catch (error) {
        console.error('Detailed error loading cigars:', error);
        console.error('Error stack:', error.stack);
        showToast('Failed to load cigars', 'error');
        elements.emptyState.style.display = 'block';
    } finally {
        elements.loading.style.display = 'none';
    }
}

// Organizer Functions
async function loadOrganizers() {
    try {
        console.log('Starting to load organizers...');
        const results = await Promise.all([
            OrganizerAPI.getBrands().catch(e => { console.error('Brands error:', e); throw e; }),
            OrganizerAPI.getSizes().catch(e => { console.error('Sizes error:', e); throw e; }),
            OrganizerAPI.getOrigins().catch(e => { console.error('Origins error:', e); throw e; }),
            OrganizerAPI.getStrengths().catch(e => { console.error('Strengths error:', e); throw e; }),
            OrganizerAPI.getRingGauges().catch(e => { console.error('Ring gauges error:', e); throw e; })
        ]);
        
        [brands, sizes, origins, strengths, ringGauges] = results;
        console.log('Organizers loaded:', { brands: brands.length, sizes: sizes.length, origins: origins.length, strengths: strengths.length, ringGauges: ringGauges.length });
        
        // Update navigation counts
        updateOrganizerCounts();
        
        // Update form dropdowns
        updateFormDropdowns();
    } catch (error) {
        console.error('Error loading organizers:', error);
        console.error('Error details:', error.message, error.stack);
        showToast('Failed to load organizers: ' + error.message, 'error');
    }
}

function updateOrganizerCounts() {
    const brandCountEl = document.getElementById('brandCount');
    const sizeCountEl = document.getElementById('sizeCount');
    const originCountEl = document.getElementById('originCount');
    const strengthCountEl = document.getElementById('strengthCount');
    const ringGaugeCountEl = document.getElementById('ringGaugeCount');
    
    if (brandCountEl) brandCountEl.textContent = brands.length;
    if (sizeCountEl) sizeCountEl.textContent = sizes.length;
    if (originCountEl) originCountEl.textContent = origins.length;
    if (strengthCountEl) strengthCountEl.textContent = strengths.length;
    if (ringGaugeCountEl) ringGaugeCountEl.textContent = ringGauges.length;
}

function updateFormDropdowns() {
    // Update brand dropdown
    const brandSelect = document.getElementById('brand');
    if (brandSelect && brandSelect.tagName === 'SELECT') {
        brandSelect.innerHTML = '<option value="">Select Brand</option>';
        brands.forEach(brand => {
            const option = document.createElement('option');
            option.value = brand.name;
            option.textContent = brand.name;
            brandSelect.appendChild(option);
        });
    }

    // Update size dropdown
    const sizeSelect = document.getElementById('size');
    if (sizeSelect && sizeSelect.tagName === 'SELECT') {
        sizeSelect.innerHTML = '<option value="">Select Size</option>';
        sizes.forEach(size => {
            const option = document.createElement('option');
            option.value = size.name;
            option.textContent = size.name;
            sizeSelect.appendChild(option);
        });
    }

    // Update origin dropdown
    const originSelect = document.getElementById('origin');
    if (originSelect && originSelect.tagName === 'SELECT') {
        originSelect.innerHTML = '<option value="">Select Origin</option>';
        origins.forEach(origin => {
            const option = document.createElement('option');
            option.value = origin.name;
            option.textContent = origin.name;
            originSelect.appendChild(option);
        });
    }

    // Update strength dropdown - this should already be a select
    const strengthSelect = document.getElementById('strength');
    if (strengthSelect) {
        strengthSelect.innerHTML = '<option value="">Select Strength</option>';
        strengths.forEach(strength => {
            const option = document.createElement('option');
            option.value = strength.name;
            option.textContent = strength.name;
            strengthSelect.appendChild(option);
        });
    }
}

// Generic organizer rendering function
function renderOrganizers(organizers, containerId, type) {
    const container = document.getElementById(containerId);
    if (!container) return;

    if (organizers.length === 0) {
        container.innerHTML = '';
        document.getElementById(`${type}EmptyState`).style.display = 'block';
        return;
    }

    document.getElementById(`${type}EmptyState`).style.display = 'none';
    
    container.innerHTML = organizers.map(organizer => 
        createOrganizerCard(organizer, type)
    ).join('');
}

function createOrganizerCard(organizer, type) {
    const getTypeIcon = (type) => {
        const icons = {
            'brands': '🏷️',
            'sizes': '📏',
            'origins': '🌍',
            'strengths': '', // Will use custom strength meter
            'ringGauges': '⭕'
        };
        return icons[type] || '📋';
    };

    const getStrengthMeter = (level) => {
        if (!level) return '';
        let bars = '';
        for (let i = 1; i <= 5; i++) {
            bars += `<span class="strength-bar${i <= level ? ' filled' : ''}"></span>`;
        }
        return `<span class="strength-meter" title="Level ${level}/5">${bars}</span>`;
    };

    const getDisplayValue = (organizer, type) => {
        switch(type) {
            case 'brands':
                return organizer.name;
            case 'sizes':
                return organizer.name;
            case 'origins':
                return organizer.name;
            case 'strengths':
                return organizer.name;
            case 'ringGauges':
                return `Ring Gauge ${organizer.gauge}`;
            default:
                return organizer.name || organizer.gauge;
        }
    };

    const getMetadata = (organizer, type) => {
        if (type === 'brands' && organizer.country) {
            return `<p class="organizer-metadata">📍 ${organizer.country}</p>`;
        }
        if (type === 'sizes' && organizer.length_inches) {
            return `<p class="organizer-metadata">${organizer.length_inches}" length</p>`;
        }
        if (type === 'strengths' && organizer.level) {
            return `<p class="organizer-metadata strength-level">${getStrengthMeter(organizer.level)} Level ${organizer.level}/5</p>`;
        }
        if (type === 'ringGauges' && organizer.common_names && organizer.common_names.length > 0) {
            return `<p class="organizer-metadata">Common: ${organizer.common_names.join(', ')}</p>`;
        }
        return '';
    };

    return `
        <div class="organizer-card" data-id="${organizer.id}">
            <div class="organizer-header">
                <span class="organizer-icon">${getTypeIcon(type)}</span>
                <h3 class="organizer-name">${getDisplayValue(organizer, type)}</h3>
                <div class="organizer-actions">
                    <button class="action-btn edit-btn" onclick="editOrganizer('${organizer.id}', '${type}')" title="Edit">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteOrganizer('${organizer.id}', '${type}')" title="Delete">🗑️</button>
                </div>
            </div>
            ${getMetadata(organizer, type)}
            ${organizer.description ? `<p class="organizer-description">${organizer.description}</p>` : ''}
        </div>
    `;
}

// Modal functions for organizers
function openOrganizerModal(type, organizer = null) {
    const modal = document.getElementById(`${type}Modal`);
    const form = document.getElementById(`${type}Form`);
    const title = document.getElementById(`${type}ModalTitle`);
    
    if (!modal || !form || !title) return;

    isEditingOrganizer = !!organizer;
    currentOrganizer = organizer;

    title.textContent = isEditingOrganizer ? `Edit ${type.slice(0, -1)}` : `Add New ${type.slice(0, -1)}`;
    
    if (isEditingOrganizer) {
        populateOrganizerForm(type, organizer);
    } else {
        form.reset();
    }

    modal.classList.add('show');
}

function populateOrganizerForm(type, organizer) {
    const form = document.getElementById(`${type}Form`);
    if (!form || !organizer) return;

    // Populate based on type
    switch(type) {
        case 'brand':
            if (organizer.name) form.querySelector('[name="name"]').value = organizer.name;
            if (organizer.country) form.querySelector('[name="country"]').value = organizer.country;
            if (organizer.website) form.querySelector('[name="website"]').value = organizer.website;
            if (organizer.description) form.querySelector('[name="description"]').value = organizer.description;
            break;
        case 'size':
            if (organizer.name) form.querySelector('[name="name"]').value = organizer.name;
            if (organizer.length_inches) form.querySelector('[name="length"]').value = organizer.length_inches;
            if (organizer.description) form.querySelector('[name="description"]').value = organizer.description;
            break;
        case 'origin':
            if (organizer.name) form.querySelector('[name="name"]').value = organizer.name;
            if (organizer.description) form.querySelector('[name="description"]').value = organizer.description;
            break;
        case 'strength':
            if (organizer.name) form.querySelector('[name="name"]').value = organizer.name;
            if (organizer.level) form.querySelector('[name="level"]').value = organizer.level;
            if (organizer.description) form.querySelector('[name="description"]').value = organizer.description;
            break;
        case 'ringGauge':
            if (organizer.gauge) form.querySelector('[name="gauge"]').value = organizer.gauge;
            if (organizer.common_names && organizer.common_names.length > 0) {
                form.querySelector('[name="common_names"]').value = organizer.common_names.join(', ');
            }
            if (organizer.description) form.querySelector('[name="description"]').value = organizer.description;
            break;
        default:
            // Generic: only populate name field
            const nameField = form.querySelector('[name="name"]');
            if (nameField && organizer.name) {
                nameField.value = organizer.name;
            }
    }
}

function closeOrganizerModal(type) {
    const modal = document.getElementById(`${type}Modal`);
    if (modal) {
        modal.classList.remove('show');
        isEditingOrganizer = false;
        currentOrganizer = null;
    }
}

async function saveOrganizer(type) {
    const form = document.getElementById(`${type}Form`);
    if (!form) return;
    
    const formData = new FormData(form);
    const data = {};
    
    // Handle name field for most types
    if (type !== 'ringGauge' && formData.get('name')) {
        data.name = formData.get('name');
    }
    
    // Add additional fields based on type
    switch(type) {
        case 'brand':
            if (formData.get('country')) data.country = formData.get('country');
            if (formData.get('website')) data.website = formData.get('website');
            if (formData.get('description')) data.description = formData.get('description');
            break;
        case 'size':
            if (formData.get('length')) data.length_inches = parseFloat(formData.get('length'));
            if (formData.get('description')) data.description = formData.get('description');
            break;
        case 'origin':
            // For origins, the country is the same as the name (country of origin)
            data.country = formData.get('name');
            if (formData.get('description')) data.description = formData.get('description');
            break;
        case 'strength':
            if (formData.get('level')) data.level = parseInt(formData.get('level'));
            if (formData.get('description')) data.description = formData.get('description');
            break;
        case 'ringGauge':
            if (formData.get('gauge')) data.gauge = parseInt(formData.get('gauge'));
            // Parse comma-separated common names into an array
            if (formData.get('common_names')) {
                const namesStr = formData.get('common_names').trim();
                if (namesStr) {
                    data.common_names = namesStr.split(',').map(n => n.trim()).filter(n => n.length > 0);
                }
            }
            if (formData.get('description')) data.description = formData.get('description');
            break;
    }
    
    try {
        // API method mapping
        const apiMethods = {
            'brand': { create: OrganizerAPI.createBrand, update: OrganizerAPI.updateBrand },
            'size': { create: OrganizerAPI.createSize, update: OrganizerAPI.updateSize },
            'origin': { create: OrganizerAPI.createOrigin, update: OrganizerAPI.updateOrigin },
            'strength': { create: OrganizerAPI.createStrength, update: OrganizerAPI.updateStrength },
            'ringGauge': { create: OrganizerAPI.createRingGauge, update: OrganizerAPI.updateRingGauge }
        };
        
        if (isEditingOrganizer && currentOrganizer) {
            await apiMethods[type].update(currentOrganizer.id, data);
            showToast(`${type.charAt(0).toUpperCase() + type.slice(1)} updated successfully!`);
        } else {
            await apiMethods[type].create(data);
            showToast(`${type.charAt(0).toUpperCase() + type.slice(1)} created successfully!`);
        }
        
        closeOrganizerModal(type);
        await loadOrganizers();
        
        // Navigate to the organizer page to show the new/updated entry
        const currentPageMap = {
            'brand': 'brands',
            'size': 'sizes',
            'origin': 'origins',
            'strength': 'strength',
            'ringGauge': 'ring-gauge'
        };
        
        const targetPage = currentPageMap[type];
        if (targetPage) {
            // Always navigate to refresh the view with new data
            navigateToPage(targetPage);
        }
        
    } catch (error) {
        console.error(`Error saving ${type}:`, error);
        showToast(error.message || `Failed to save ${type}`, 'error');
    }
}

// Import URL Modal Functions
function openImportUrlModal() {
    const modal = document.getElementById('importUrlModal');
    modal.classList.add('show');
    document.getElementById('importUrl').value = '';
    document.getElementById('importStatus').innerHTML = '';
    document.getElementById('importDestination').style.display = 'none';
    document.getElementById('startImportBtn').style.display = 'inline-block';
    document.getElementById('confirmImportBtn').style.display = 'none';
    scrapedCigarData = null;
    document.getElementById('importUrl').focus();
}

function closeImportUrlModal() {
    const modal = document.getElementById('importUrlModal');
    modal.classList.remove('show');
    scrapedCigarData = null;
}

async function importFromUrl() {
    const urlInput = document.getElementById('importUrl');
    const statusDiv = document.getElementById('importStatus');
    const importBtn = document.getElementById('startImportBtn');
    
    const url = urlInput.value.trim();
    
    if (!url) {
        statusDiv.innerHTML = '<p class="error-message">Please enter a URL</p>';
        return;
    }
    
    // Validate URL format
    try {
        new URL(url);
    } catch (e) {
        statusDiv.innerHTML = '<p class="error-message">Please enter a valid URL</p>';
        return;
    }
    
    // Show loading state
    importBtn.disabled = true;
    statusDiv.innerHTML = '<p class="loading-message"><i class="mdi mdi-loading mdi-spin"></i> Scraping cigar information...</p>';
    
    try {
        const response = await makeAuthenticatedRequest('/api/v1/cigars/scrape', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ url })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to scrape URL');
        }
        
        scrapedCigarData = await response.json();
        console.log('Scraped cigar data:', scrapedCigarData);
        
        // Build a summary of what was found
        let summary = '<div class="scraped-summary"><strong>Found:</strong><ul>';
        if (scrapedCigarData.brand) summary += `<li>Brand: ${scrapedCigarData.brand}</li>`;
        if (scrapedCigarData.name) summary += `<li>Name: ${scrapedCigarData.name}</li>`;
        if (scrapedCigarData.size) summary += `<li>Size: ${scrapedCigarData.size}</li>`;
        if (scrapedCigarData.ring_gauge) summary += `<li>Ring Gauge: ${scrapedCigarData.ring_gauge}</li>`;
        if (scrapedCigarData.strength) summary += `<li>Strength: ${scrapedCigarData.strength}</li>`;
        if (scrapedCigarData.origin) summary += `<li>Origin: ${scrapedCigarData.origin}</li>`;
        summary += '</ul></div>';
        
        statusDiv.innerHTML = `<p class="success-message"><i class="mdi mdi-check-circle"></i> Successfully scraped cigar information!</p>${summary}`;
        
        // Show destination selection
        document.getElementById('importDestination').style.display = 'block';
        document.getElementById('startImportBtn').style.display = 'none';
        document.getElementById('confirmImportBtn').style.display = 'inline-block';
        
        // Populate humidor dropdown
        await populateImportHumidorDropdown();
        
    } catch (error) {
        console.error('Import error:', error);
        statusDiv.innerHTML = `<p class="error-message"><i class="mdi mdi-alert-circle"></i> ${error.message}</p>`;
    } finally {
        importBtn.disabled = false;
    }
}

async function populateImportHumidorDropdown() {
    const humidorSelect = document.getElementById('importHumidor');
    humidorSelect.innerHTML = '<option value="">Select humidor...</option>';
    
    humidors.forEach(humidor => {
        const option = document.createElement('option');
        option.value = humidor.id;
        option.textContent = humidor.name;
        humidorSelect.appendChild(option);
    });
}

async function confirmImport() {
    const location = document.getElementById('importLocation').value;
    const statusDiv = document.getElementById('importStatus');
    const confirmBtn = document.getElementById('confirmImportBtn');
    
    if (!location) {
        statusDiv.innerHTML = '<p class="error-message">Please select where to add this cigar</p>';
        return;
    }
    
    if (location === 'humidor') {
        const humidorId = document.getElementById('importHumidor').value;
        if (!humidorId) {
            statusDiv.innerHTML = '<p class="error-message">Please select a humidor</p>';
            return;
        }
        
        const quantity = parseInt(document.getElementById('importQuantity').value) || 1;
        await createCigarFromScrapedData(humidorId, quantity, false);
    } else if (location === 'wishlist') {
        await createCigarFromScrapedData(null, null, true);
    }
}

async function createCigarFromScrapedData(humidorId, quantity, isWishList) {
    const statusDiv = document.getElementById('importStatus');
    const confirmBtn = document.getElementById('confirmImportBtn');
    
    try {
        confirmBtn.disabled = true;
        statusDiv.innerHTML = '<p class="loading-message"><i class="mdi mdi-loading mdi-spin"></i> Creating cigar...</p>';
        
        // Find or create organizer IDs
        const brandId = await findOrCreateOrganizer('brand', scrapedCigarData.brand);
        const sizeId = await findOrCreateOrganizer('size', scrapedCigarData.size);
        const strengthId = await findOrCreateOrganizer('strength', scrapedCigarData.strength);
        const originId = await findOrCreateOrganizer('origin', scrapedCigarData.origin);
        const ringGaugeId = await findOrCreateRingGauge(scrapedCigarData.ring_gauge);
        
        const cigarData = {
            name: scrapedCigarData.name || 'Unknown',
            brand_id: brandId,
            size_id: sizeId,
            strength_id: strengthId,
            origin_id: originId,
            ring_gauge_id: ringGaugeId,
            humidor_id: humidorId,
            quantity: quantity || null,
            is_wish_list: isWishList
        };
        
        console.log('Creating cigar with data:', cigarData);
        
        const response = await makeAuthenticatedRequest('/api/v1/cigars', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(cigarData)
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to create cigar');
        }
        
        const newCigar = await response.json();
        console.log('Created cigar:', newCigar);
        
        showToast('Cigar added successfully!', 'success');
        closeImportUrlModal();
        
        // Refresh the appropriate view
        if (isWishList) {
            await loadWishList();
        } else if (humidorId) {
            // Reload the humidor detail view to show the new cigar
            await showHumidorDetail(humidorId);
        }
        
    } catch (error) {
        console.error('Error creating cigar:', error);
        statusDiv.innerHTML = `<p class="error-message"><i class="mdi mdi-alert-circle"></i> ${error.message}</p>`;
    } finally {
        confirmBtn.disabled = false;
    }
}

async function findOrCreateOrganizer(type, name) {
    if (!name) return null;
    
    // Map type to collection and endpoint
    const typeMap = {
        'brand': { collection: brands, endpoint: 'brands' },
        'size': { collection: sizes, endpoint: 'sizes' },
        'strength': { collection: strengths, endpoint: 'strengths' },
        'origin': { collection: origins, endpoint: 'origins' }
    };
    
    const config = typeMap[type];
    if (!config) return null;
    
    // Check if it already exists (case-insensitive)
    const existing = config.collection.find(
        item => item.name.toLowerCase() === name.toLowerCase()
    );
    
    if (existing) {
        console.log(`Found existing ${type}:`, existing.name, '-> ID:', existing.id);
        return existing.id;
    }
    
    // Create new organizer
    try {
        console.log(`Creating new ${type}:`, name);
        const response = await makeAuthenticatedRequest(`/api/v1/${config.endpoint}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ name })
        });
        
        if (!response.ok) {
            throw new Error(`Failed to create ${type}`);
        }
        
        const newOrganizer = await response.json();
        console.log(`Created ${type}:`, newOrganizer.name, '-> ID:', newOrganizer.id);
        
        // Add to local collection
        config.collection.push(newOrganizer);
        
        return newOrganizer.id;
    } catch (error) {
        console.error(`Error creating ${type}:`, error);
        return null;
    }
}

async function findOrCreateRingGauge(gaugeValue) {
    if (!gaugeValue) return null;
    
    // Parse the gauge number
    const gaugeNum = parseInt(gaugeValue);
    if (isNaN(gaugeNum)) return null;
    
    // Validate range (backend requires 20-100)
    if (gaugeNum < 20 || gaugeNum > 100) {
        console.log(`Ring gauge ${gaugeNum} is outside valid range (20-100), skipping`);
        return null;
    }
    
    // Check if it already exists
    const existing = ringGauges.find(rg => rg.gauge === gaugeNum);
    
    if (existing) {
        console.log('Found existing ring gauge:', existing.gauge, '-> ID:', existing.id);
        return existing.id;
    }
    
    // Create new ring gauge
    try {
        console.log('Creating new ring gauge:', gaugeNum);
        const response = await makeAuthenticatedRequest('/api/v1/ring-gauges', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ gauge: gaugeNum })
        });
        
        if (!response.ok) {
            throw new Error('Failed to create ring gauge');
        }
        
        const newRingGauge = await response.json();
        console.log('Created ring gauge:', newRingGauge.gauge, '-> ID:', newRingGauge.id);
        
        // Add to local collection
        ringGauges.push(newRingGauge);
        
        return newRingGauge.id;
    } catch (error) {
        console.error('Error creating ring gauge:', error);
        return null;
    }
}

// Event Listeners
document.addEventListener('DOMContentLoaded', function() {
    // Check if this is a public share link
    const isPublicShare = window.location.pathname.startsWith('/shared/humidors/');
    
    if (isPublicShare) {
        // Handle public share view
        initializePublicShareView();
        return;
    }
    
    // Check authentication first for regular app
    if (!checkAuth()) {
        window.location.href = '/login.html';
        return;
    }
    
    // Initialize user info display
    initializeUserDisplay();
    
    // Initialize theme
    initializeTheme();
    setupThemeToggle();
    
    // Initialize PWA
    registerServiceWorker();
    setupInstallPrompt();
    
    // Initialize DOM elements
    elements = {
        welcomeSection: document.getElementById('welcomeSection'),
        mainContentSection: document.getElementById('mainContentSection'),
        createFirstHumidorBtn: document.getElementById('createFirstHumidorBtn'),
        addHumidorBtn: document.getElementById('addHumidorBtn'),
        addHumidorBtnSidebar: document.getElementById('addHumidorBtnSidebar'),
        addHumidorBtnHeader: document.getElementById('addHumidorBtnHeader'),
        addCigarBtn: document.getElementById('addCigarBtn'),
        addCigarBtnNav: document.getElementById('addCigarBtnNav'),
        humidorsContainer: document.getElementById('humidorsContainer'),
        collectionSummary: document.getElementById('collectionSummary'),
        
        // Modals
        humidorModal: document.getElementById('humidorModal'),
        humidorForm: document.getElementById('humidorForm'),
        closeHumidorModal: document.getElementById('closeHumidorModal'),
        cancelHumidorBtn: document.getElementById('cancelHumidorBtn'),
        saveHumidorBtn: document.getElementById('saveHumidorBtn'),
        
        cigarModal: document.getElementById('cigarModal'),
        cigarForm: document.getElementById('cigarForm'),
        closeCigarModal: document.getElementById('closeCigarModal'),
        cancelCigarBtn: document.getElementById('cancelCigarBtn'),
        saveCigarBtn: document.getElementById('saveCigarBtn'),
        
        toastContainer: document.getElementById('toastContainer')
    };

    // Initialize navigation and router
    initializeNavigation();
    initializeRouter();
    
    // Initialize mobile menu
    initializeMobileMenu();
    
    // Initialize mobile filter toggle
    initializeMobileFilterToggle();
    
    // Event listeners for new interface
    if (elements.createFirstHumidorBtn) {
        elements.createFirstHumidorBtn.addEventListener('click', () => {
            closeMobileMenu();
            openHumidorModal();
        });
    }
    if (elements.addHumidorBtn) {
        elements.addHumidorBtn.addEventListener('click', () => {
            closeMobileMenu();
            openHumidorModal();
        });
    }
    if (elements.addHumidorBtnSidebar) {
        elements.addHumidorBtnSidebar.addEventListener('click', () => {
            closeMobileMenu();
            openHumidorModal();
        });
    }
    if (elements.addHumidorBtnHeader) {
        elements.addHumidorBtnHeader.addEventListener('click', () => {
            closeMobileMenu();
            openHumidorModal();
        });
    }
    if (elements.addCigarBtn) {
        elements.addCigarBtn.addEventListener('click', () => {
            closeMobileMenu();
            openCigarModal();
        });
    }
    if (elements.addCigarBtnNav) {
        elements.addCigarBtnNav.addEventListener('click', () => {
            closeMobileMenu();
            openCigarModal();
        });
    }
    
    // User dropdown menu toggle
    const userMenuTrigger = document.getElementById('userMenuTrigger');
    const userDropdownMenu = document.getElementById('userDropdownMenu');
    
    if (userMenuTrigger && userDropdownMenu) {
        userMenuTrigger.addEventListener('click', (e) => {
            e.stopPropagation();
            userDropdownMenu.classList.toggle('active');
        });
        
        // Close dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!userDropdownMenu.contains(e.target) && !userMenuTrigger.contains(e.target)) {
                userDropdownMenu.classList.remove('active');
            }
        });
    }
    
    // Logout button event
    const logoutBtn = document.getElementById('logoutBtn');
    if (logoutBtn) {
        logoutBtn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            logout();
        });
    }
    
    // Recommend button event
    const recommendBtn = document.getElementById('recommendCigarBtn');
    if (recommendBtn) {
        recommendBtn.addEventListener('click', function(e) {
            e.preventDefault();
            openRecommendModal();
        });
    }
    
    // Close recommendation modal on backdrop click
    const recommendModal = document.getElementById('recommendModal');
    if (recommendModal) {
        recommendModal.addEventListener('click', function(e) {
            if (e.target === recommendModal) {
                closeRecommendModal();
            }
        });
    }
    
    // Humidor modal events
    if (elements.closeHumidorModal) {
        elements.closeHumidorModal.addEventListener('click', closeHumidorModal);
    }
    if (elements.cancelHumidorBtn) {
        elements.cancelHumidorBtn.addEventListener('click', closeHumidorModal);
    }
    if (elements.humidorForm) {
        elements.humidorForm.addEventListener('submit', (e) => {
            e.preventDefault();
            saveHumidor();
        });
    }
    
    // Report Card modal events
    const closeReportCardBtn = document.getElementById('closeReportCardModal');
    if (closeReportCardBtn) {
        closeReportCardBtn.addEventListener('click', closeReportCard);
    }
    
    // Transfer modal events
    const closeTransferBtn = document.getElementById('closeTransferModal');
    if (closeTransferBtn) {
        console.log('[TRANSFER] Close button found, attaching listener');
        closeTransferBtn.addEventListener('click', (e) => {
            console.log('[TRANSFER] Close button clicked');
            e.preventDefault();
            e.stopPropagation();
            closeTransferModal();
        });
    } else {
        console.warn('[TRANSFER] Close button NOT found');
    }
    // Note: Transfer cigar modal uses inline onclick handlers, not event listeners
    const transferAllBtn = document.getElementById('transferAllBtn');
    if (transferAllBtn) {
        transferAllBtn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (currentTransferCigar) {
                document.getElementById('transferQuantity').value = currentTransferCigar.quantity;
            }
        });
    }
    
    // Transfer modal click outside to close
    const transferModal = document.getElementById('transferCigarModal');
    if (transferModal) {
        transferModal.addEventListener('click', (e) => {
            if (e.target === transferModal) {
                closeTransferModal();
            }
        });
    }
    
    // Import URL modal events
    const importFromUrlBtn = document.getElementById('importFromUrlBtn');
    const closeImportUrlBtn = document.getElementById('closeImportUrlModal');
    const startImportBtn = document.getElementById('startImportBtn');
    const confirmImportBtn = document.getElementById('confirmImportBtn');
    const importLocation = document.getElementById('importLocation');
    const importUrlModal = document.getElementById('importUrlModal');
    const importUrlInput = document.getElementById('importUrl');
    
    if (importFromUrlBtn) {
        importFromUrlBtn.addEventListener('click', openImportUrlModal);
    }
    if (closeImportUrlBtn) {
        closeImportUrlBtn.addEventListener('click', closeImportUrlModal);
    }
    if (startImportBtn) {
        startImportBtn.addEventListener('click', importFromUrl);
    }
    if (confirmImportBtn) {
        confirmImportBtn.addEventListener('click', confirmImport);
    }
    if (importLocation) {
        importLocation.addEventListener('change', (e) => {
            const humidorGroup = document.getElementById('importHumidorGroup');
            const quantityGroup = document.getElementById('importQuantityGroup');
            
            if (e.target.value === 'humidor') {
                humidorGroup.style.display = 'block';
                quantityGroup.style.display = 'block';
            } else if (e.target.value === 'wishlist') {
                humidorGroup.style.display = 'none';
                quantityGroup.style.display = 'none';
            } else {
                humidorGroup.style.display = 'none';
                quantityGroup.style.display = 'none';
            }
        });
    }
    if (importUrlModal) {
        importUrlModal.addEventListener('click', (e) => {
            if (e.target === importUrlModal) {
                closeImportUrlModal();
            }
        });
    }
    if (importUrlInput) {
        importUrlInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                importFromUrl();
            }
        });
    }
    
    // Image upload button handler
    const uploadImageBtn = document.getElementById('uploadImageBtn');
    const cigarImageUpload = document.getElementById('cigarImageUpload');
    const uploadFileName = document.getElementById('uploadFileName');
    
    if (uploadImageBtn && cigarImageUpload) {
        uploadImageBtn.addEventListener('click', () => {
            cigarImageUpload.click();
        });
        
        cigarImageUpload.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                uploadFileName.textContent = `Selected: ${file.name}`;
            } else {
                uploadFileName.textContent = 'JPG or PNG, max 5MB';
            }
        });
    }
    
    // Humidor image upload button handler
    const uploadHumidorImageBtn = document.getElementById('uploadHumidorImageBtn');
    const humidorImageUpload = document.getElementById('humidorImageUpload');
    const uploadHumidorFileName = document.getElementById('uploadHumidorFileName');
    
    if (uploadHumidorImageBtn && humidorImageUpload) {
        uploadHumidorImageBtn.addEventListener('click', () => {
            humidorImageUpload.click();
        });
        
        humidorImageUpload.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                uploadHumidorFileName.textContent = `Selected: ${file.name}`;
            } else {
                uploadHumidorFileName.textContent = 'JPG or PNG, max 5MB';
            }
        });
    }
    
    // Humidor dropdown change handler for wish list
    const cigarHumidorSelect = document.getElementById('cigarHumidor');
    const cigarQuantityInput = document.getElementById('cigarQuantity');
    const cigarPurchaseDateInput = document.getElementById('cigarPurchaseDate');
    
    if (cigarHumidorSelect) {
        cigarHumidorSelect.addEventListener('change', (e) => {
            const isWishList = e.target.value === 'WISH_LIST';
            
            if (isWishList) {
                // Disable and clear quantity and purchase date for wish list
                cigarQuantityInput.disabled = true;
                cigarQuantityInput.value = '';
                cigarQuantityInput.removeAttribute('required');
                cigarPurchaseDateInput.disabled = true;
                cigarPurchaseDateInput.value = '';
                
                // Add visual indicator with lighter styling
                cigarQuantityInput.parentElement.style.opacity = '0.6';
                cigarQuantityInput.parentElement.style.pointerEvents = 'none';
                cigarPurchaseDateInput.parentElement.style.opacity = '0.6';
                cigarPurchaseDateInput.parentElement.style.pointerEvents = 'none';
            } else {
                // Enable for regular humidors
                cigarQuantityInput.disabled = false;
                cigarQuantityInput.value = '1';
                cigarQuantityInput.setAttribute('required', 'required');
                cigarPurchaseDateInput.disabled = false;
                
                // Remove visual indicator
                cigarQuantityInput.parentElement.style.opacity = '1';
                cigarQuantityInput.parentElement.style.pointerEvents = 'auto';
                cigarPurchaseDateInput.parentElement.style.opacity = '1';
                cigarPurchaseDateInput.parentElement.style.pointerEvents = 'auto';
            }
        });
    }
    
    // Cigar modal events
    if (elements.closeCigarModal) {
        elements.closeCigarModal.addEventListener('click', closeCigarModal);
    }
    if (elements.cancelCigarBtn) {
        elements.cancelCigarBtn.addEventListener('click', closeCigarModal);
    }
    if (elements.cigarForm) {
        elements.cigarForm.addEventListener('submit', (e) => {
            e.preventDefault();
            saveCigar();
        });
    }
    
    // Close modals when clicking outside
    if (elements.humidorModal) {
        elements.humidorModal.addEventListener('click', function(event) {
            if (event.target === elements.humidorModal) {
                closeHumidorModal();
            }
        });
    }
    if (elements.cigarModal) {
        elements.cigarModal.addEventListener('click', function(event) {
            if (event.target === elements.cigarModal) {
                closeCigarModal();
            }
        });
    }
    
    // Organizer modal events
    const organizerTypes = ['brand', 'size', 'origin', 'strength', 'ringGauge'];
    organizerTypes.forEach(type => {
        const modal = document.getElementById(`${type}Modal`);
        const form = document.getElementById(`${type}Form`);
        const closeBtn = document.getElementById(`close${type.charAt(0).toUpperCase() + type.slice(1)}Modal`);
        const cancelBtn = document.getElementById(`cancel${type.charAt(0).toUpperCase() + type.slice(1)}Btn`);
        
        // Form submission
        if (form) {
            form.addEventListener('submit', async (e) => {
                e.preventDefault();
                await saveOrganizer(type);
            });
        }
        
        // Close button
        if (closeBtn) {
            closeBtn.addEventListener('click', () => closeOrganizerModal(type));
        }
        
        // Cancel button
        if (cancelBtn) {
            cancelBtn.addEventListener('click', (e) => {
                e.preventDefault();
                closeOrganizerModal(type);
            });
        }
        
        // Click outside to close
        if (modal) {
            modal.addEventListener('click', function(event) {
                if (event.target === modal) {
                    closeOrganizerModal(type);
                }
            });
        }
    });
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(event) {
        if (event.key === 'Escape') {
            closeHumidorModal();
            closeCigarModal();
            closeTransferModal();
            closeReportCard();
            // Close all organizer modals
            organizerTypes.forEach(type => closeOrganizerModal(type));
        }
        if (event.key === 'h' && (event.ctrlKey || event.metaKey)) {
            event.preventDefault();
            openHumidorModal();
        }
        if (event.key === 'c' && (event.ctrlKey || event.metaKey)) {
            event.preventDefault();
            openCigarModal();
        }
    });
    
    // Search and Filter Event Listeners
    const cigarSearchInput = document.getElementById('cigarSearchInput');
    const cigarSearchBtn = document.getElementById('cigarSearchBtn');
    const clearFiltersBtn = document.getElementById('clearFiltersBtn');
    
    if (cigarSearchInput) {
        cigarSearchInput.addEventListener('input', (e) => {
            searchQuery = e.target.value;
            applySearchAndFilters();
        });
    }
    
    if (cigarSearchBtn) {
        cigarSearchBtn.addEventListener('click', () => {
            applySearchAndFilters();
        });
    }
    
    if (clearFiltersBtn) {
        clearFiltersBtn.addEventListener('click', clearFilters);
    }
    
    // Filter chip event listeners
    document.getElementById('filterBrandChip')?.addEventListener('click', () => openFilterModal('brand'));
    document.getElementById('filterSizeChip')?.addEventListener('click', () => openFilterModal('size'));
    document.getElementById('filterOriginChip')?.addEventListener('click', () => openFilterModal('origin'));
    document.getElementById('filterStrengthChip')?.addEventListener('click', () => openFilterModal('strength'));
    document.getElementById('filterRingGaugeChip')?.addEventListener('click', () => openFilterModal('ringGauge'));
    
    // Filter modal event listeners
    const filterModal = document.getElementById('filterModal');
    const closeFilterModalBtn = document.getElementById('closeFilterModal');
    const applyFilterBtn = document.getElementById('applyFilterSelection');
    const clearFilterBtn = document.getElementById('clearFilterSelection');
    const filterSearchInput = document.getElementById('filterSearchInput');
    
    if (closeFilterModalBtn) {
        closeFilterModalBtn.addEventListener('click', closeFilterModal);
    }
    
    if (applyFilterBtn) {
        applyFilterBtn.addEventListener('click', applyFilterSelection);
    }
    
    if (clearFilterBtn) {
        clearFilterBtn.addEventListener('click', clearFilterSelection);
    }
    
    if (filterSearchInput) {
        filterSearchInput.addEventListener('input', (e) => filterSearchHandler(e.target.value));
    }
    
    if (filterModal) {
        filterModal.addEventListener('click', (e) => {
            if (e.target === filterModal) {
                closeFilterModal();
            }
        });
    }
    
    // Pagination event listeners
    const firstPageBtn = document.getElementById('firstPageBtn');
    const prevPageBtn = document.getElementById('prevPageBtn');
    const nextPageBtn = document.getElementById('nextPageBtn');
    const lastPageBtn = document.getElementById('lastPageBtn');
    const pageSizeSelect = document.getElementById('pageSizeSelect');
    
    if (firstPageBtn) {
        firstPageBtn.addEventListener('click', firstPage);
    }
    
    if (prevPageBtn) {
        prevPageBtn.addEventListener('click', previousPage);
    }
    
    if (nextPageBtn) {
        nextPageBtn.addEventListener('click', nextPage);
    }
    
    if (lastPageBtn) {
        lastPageBtn.addEventListener('click', lastPage);
    }
    
    if (pageSizeSelect) {
        pageSizeSelect.addEventListener('change', (e) => {
            changePageSize(e.target.value);
        });
    }
    
    // Initialize the interface
    // Note: loadHumidors() will call loadOrganizers() internally first
    loadHumidors();

    // Dropdown functionality
    initializeDropdowns();
});

// Dropdown Functions
function initializeDropdowns() {
    const dropdownToggle = document.getElementById('organizersToggle');
    const dropdownContent = document.getElementById('organizersDropdown');
    const dropdown = dropdownToggle?.parentElement;

    if (dropdownToggle && dropdownContent && dropdown) {
        dropdownToggle.addEventListener('click', function(e) {
            e.preventDefault();
            toggleDropdown(dropdown);
        });

        // Close dropdown when clicking outside
        document.addEventListener('click', function(e) {
            if (!dropdown.contains(e.target)) {
                closeDropdown(dropdown);
            }
        });
    }
}

function toggleDropdown(dropdown) {
    const isOpen = dropdown.classList.contains('open');
    
    // Close all dropdowns first
    document.querySelectorAll('.nav-dropdown.open').forEach(d => {
        d.classList.remove('open');
        const toggle = d.querySelector('.nav-dropdown-toggle');
        if (toggle) toggle.classList.remove('active');
    });

    // Toggle current dropdown
    if (!isOpen) {
        dropdown.classList.add('open');
        const toggle = dropdown.querySelector('.nav-dropdown-toggle');
        if (toggle) toggle.classList.add('active');
    }
}

function closeDropdown(dropdown) {
    dropdown.classList.remove('open');
    const toggle = dropdown.querySelector('.nav-dropdown-toggle');
    if (toggle) toggle.classList.remove('active');
}

// Navigation Functions
function initializeNavigation() {
    // Add event listeners to navigation items
    document.querySelectorAll('.nav-item').forEach(item => {
        item.addEventListener('click', (e) => {
            // Allow external links (like settings.html) to work normally
            const href = item.getAttribute('href');
            if (href && (href.startsWith('/static/') || href.startsWith('http'))) {
                return; // Let the browser handle it
            }
            
            e.preventDefault();
            const page = item.getAttribute('data-page');
            navigateToPage(page);
        });
    });

    // Add event listeners to action cards
    document.getElementById('addHumidorCard')?.addEventListener('click', () => openHumidorModal());
    document.getElementById('viewBrandsCard')?.addEventListener('click', () => navigateToPage('brands'));
    document.getElementById('humidorManagementCard')?.addEventListener('click', () => navigateToPage('humidors'));

    // Add event listeners to organizer buttons
    document.getElementById('addBrandBtn')?.addEventListener('click', () => openBrandModal());
    document.getElementById('addSizeBtn')?.addEventListener('click', () => openSizeModal());
    document.getElementById('addOriginBtn')?.addEventListener('click', () => openOriginModal());
    document.getElementById('addStrengthBtn')?.addEventListener('click', () => openStrengthModal());
    document.getElementById('addRingGaugeBtn')?.addEventListener('click', () => openRingGaugeModal());
    
    // Add event listeners for profile/settings buttons
    document.getElementById('saveProfileBtn')?.addEventListener('click', saveProfile);
    document.getElementById('changePasswordBtn')?.addEventListener('click', changePassword);
    
    // Add event listener for Account Settings in user dropdown
    document.querySelectorAll('[data-page="profile"]').forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            navigateToPage('profile');
            // Close the user dropdown
            document.getElementById('userDropdownMenu')?.classList.remove('show');
        });
    });
}

function navigateToPage(page) {
    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.toggle('active', item.getAttribute('data-page') === page);
    });

    // Hide all sections
    document.querySelectorAll('.humidors-section, .organizer-section, .profile-section, .app-settings-section, .favorites-section, .wishlist-section').forEach(section => {
        section.style.display = 'none';
    });

    // Show appropriate section and load data
    switch (page) {
        case 'humidors':
            document.getElementById('humidorsSection').style.display = 'block';
            loadHumidors();
            break;
        case 'favorites':
            document.getElementById('favoritesSection').style.display = 'block';
            loadFavorites();
            break;
        case 'wishlist':
            document.getElementById('wishlistSection').style.display = 'block';
            loadWishList();
            break;
        case 'brands':
            document.getElementById('brandsSection').style.display = 'block';
            renderOrganizers(brands, 'brandsGrid', 'brands');
            break;
        case 'sizes':
            document.getElementById('sizesSection').style.display = 'block';
            renderOrganizers(sizes, 'sizesGrid', 'sizes');
            break;
        case 'origins':
            document.getElementById('originsSection').style.display = 'block';
            renderOrganizers(origins, 'originsGrid', 'origins');
            break;
        case 'strength':
            document.getElementById('strengthSection').style.display = 'block';
            renderOrganizers(strengths, 'strengthsGrid', 'strengths');
            break;
        case 'ring-gauge':
            document.getElementById('ringGaugeSection').style.display = 'block';
            renderOrganizers(ringGauges, 'ringGaugesGrid', 'ringGauges');
            break;
        case 'profile':
            document.getElementById('profileSection').style.display = 'block';
            loadUserProfile();
            break;
        case 'settings':
            document.getElementById('settingsSection').style.display = 'block';
            initializeBackupHandlers();
            initializeUserManagementHandlers();
            loadBackups();
            // Show/hide user management section based on admin status
            const userMgmtSection = document.getElementById('userManagementSection');
            if (userMgmtSection) {
                if (isCurrentUserAdmin()) {
                    userMgmtSection.style.display = 'block';
                    loadUsers();
                } else {
                    userMgmtSection.style.display = 'none';
                }
            }
            break;
    }

    currentPage = page;
}

// Organizer action functions
function editOrganizer(id, type) {
    const typeMap = {
        'brands': brands,
        'sizes': sizes,
        'origins': origins,
        'strengths': strengths,
        'ringGauges': ringGauges
    };
    
    const organizer = typeMap[type]?.find(o => o.id === id);
    if (organizer) {
        const modalType = type === 'ringGauges' ? 'ringGauge' : type.slice(0, -1);
        openOrganizerModal(modalType, organizer);
    }
}

async function deleteOrganizer(id, type) {
    if (!confirm('Are you sure you want to delete this item?')) return;
    
    try {
        const apiMap = {
            'brands': OrganizerAPI.deleteBrand,
            'sizes': OrganizerAPI.deleteSize,
            'origins': OrganizerAPI.deleteOrigin,
            'strengths': OrganizerAPI.deleteStrength,
            'ringGauges': OrganizerAPI.deleteRingGauge
        };
        
        await apiMap[type](id);
        showToast('Item deleted successfully!');
        await loadOrganizers();
        navigateToPage(currentPage); // Refresh current view
    } catch (error) {
        console.error('Error deleting item:', error);
        showToast('Failed to delete item', 'error');
    }
}

// Humidor Functions
async function loadHumidors() {
    try {
        console.log('=== loadHumidors() called ===');
        
        // IMPORTANT: Load organizers FIRST before loading cigars
        console.log('→ Loading organizers first...');
        await loadOrganizers();
        console.log('✓ Organizers loaded, proceeding to load humidors');
        
        const response = await makeAuthenticatedRequest('/api/v1/humidors');

        if (response.ok) {
            humidors = await response.json();
            console.log('✓ Humidors loaded:', humidors.length, humidors);
            
            // Load cigars for each humidor
            cigars = [];
            for (const humidor of humidors) {
                console.log(`→ Loading cigars for humidor "${humidor.name}" (${humidor.id})...`);
                const cigarResponse = await makeAuthenticatedRequest(`/api/v1/cigars?humidor_id=${humidor.id}`);
                if (cigarResponse.ok) {
                    const responseData = await cigarResponse.json();
                    const humidorCigars = responseData.cigars || [];
                    console.log(`✓ Loaded ${humidorCigars.length} cigars for humidor ${humidor.name}:`, humidorCigars);
                    cigars.push(...humidorCigars);
                } else {
                    console.error(`✗ Failed to load cigars for humidor ${humidor.id}:`, cigarResponse.status);
                }
            }
            console.log('✓ Total cigars loaded:', cigars.length, cigars);
            console.log('✓ Organizers available - brands:', brands.length, 'sizes:', sizes.length, 'origins:', origins.length, 'strengths:', strengths.length, 'ringGauges:', ringGauges.length);
        } else {
            console.error('✗ Failed to load humidors:', response.status, response.statusText);
            humidors = [];
            cigars = [];
        }
        
        // Show appropriate section based on whether humidors exist
        console.log('→ Calling showAppropriateSection()');
        showAppropriateSection();
    } catch (error) {
        console.error('✗ Error loading humidors:', error);
        if (error.message !== 'Authentication failed') {
            showToast('Failed to load humidors', 'error');
        }
        // Reset arrays on error
        humidors = [];
        cigars = [];
        showWelcomeSection();
    }
}

function showAppropriateSection() {
    const welcomeSection = document.getElementById('welcomeSection');
    const mainContentSection = document.getElementById('mainContentSection');
    
    if (humidors.length === 0) {
        welcomeSection.style.display = 'block';
        mainContentSection.style.display = 'none';
    } else {
        // Let the router handle displaying the correct view
        handleRouteChange();
    }
}

function showWelcomeSection() {
    document.getElementById('welcomeSection').style.display = 'block';
    document.getElementById('mainContentSection').style.display = 'none';
}

// Humidor Hub View (for multiple humidors)
function showHumidorHub() {
    const welcomeSection = document.getElementById('welcomeSection');
    const mainContentSection = document.getElementById('mainContentSection');
    const humidorsContainer = document.getElementById('humidorsContainer');
    const paginationContainer = document.getElementById('paginationContainer');
    
    welcomeSection.style.display = 'none';
    mainContentSection.style.display = 'block';
    
    // Hide pagination on hub view
    if (paginationContainer) {
        paginationContainer.style.display = 'none';
    }
    
    // Render humidor cards
    humidorsContainer.innerHTML = `
        <div class="humidor-hub">
            <div class="humidor-hub-header">
                <p>Select a humidor to view its contents</p>
            </div>
            <div class="humidor-cards-grid">
                ${humidors.map(humidor => {
                    const humidorCigars = cigars.filter(c => c.humidor_id === humidor.id);
                    const totalQuantity = humidorCigars.reduce((sum, c) => sum + (c.quantity || 0), 0);
                    
                    // Determine image source or use placeholder
                    const imageHtml = humidor.image_url 
                        ? `<img src="${humidor.image_url}" alt="${escapeHtml(humidor.name)}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
                           <img src="/static/humidor-placeholder.png" alt="Humidor placeholder" style="display: none; width: 100%; height: 100%; object-fit: cover;">`
                        : `<img src="/static/humidor-placeholder.png" alt="Humidor placeholder" style="width: 100%; height: 100%; object-fit: cover;">`;
                    
                    return `
                        <div class="humidor-card" onclick="navigateToHumidorDetail('${humidor.id}')">
                            <div class="humidor-card-image">
                                ${imageHtml}
                                <div class="humidor-card-actions" onclick="event.stopPropagation();">
                                    <button class="btn-icon" onclick="openShareHumidorModal('${humidor.id}', '${escapeHtml(humidor.name).replace(/'/g, "\\'")}'))" title="Share Humidor">
                                        <i class="mdi mdi-share-variant"></i>
                                    </button>
                                    <button class="btn-icon" onclick="editHumidor('${humidor.id}')" title="Edit Humidor">
                                        <i class="mdi mdi-pencil"></i>
                                    </button>
                                    <button class="btn-icon" onclick="deleteHumidor('${humidor.id}')" title="Delete Humidor">
                                        <i class="mdi mdi-delete"></i>
                                    </button>
                                </div>
                            </div>
                            <div class="humidor-card-content">
                                <div class="humidor-card-header">
                                    <h3>${escapeHtml(humidor.name)}</h3>
                                    <p class="humidor-location">${humidor.location ? `<i class="mdi mdi-map-marker"></i>${escapeHtml(humidor.location)}` : ''}</p>
                                </div>
                                <div class="humidor-card-stats">
                                    <div class="stat">
                                        <i class="mdi mdi-cigar"></i>
                                        <span class="stat-value">${totalQuantity}</span>
                                        <span class="stat-label">Cigars</span>
                                    </div>
                                    <div class="stat">
                                        <i class="mdi mdi-format-list-bulleted"></i>
                                        <span class="stat-value">${humidorCigars.length}</span>
                                        <span class="stat-label">Types</span>
                                    </div>
                                </div>
                                ${humidor.notes ? `
                                    <div class="humidor-card-notes">
                                        <p>${escapeHtml(humidor.notes)}</p>
                                    </div>
                                ` : ''}
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        </div>
    `;
}

// Humidor Detail View (for single humidor with pagination)
async function showHumidorDetail(humidorId) {
    console.log('showHumidorDetail called with ID:', humidorId);
    console.log('Available humidors:', humidors);
    
    const humidor = humidors.find(h => h.id === humidorId);
    if (!humidor) {
        console.log('Humidor not found!');
        showToast('Humidor not found', 'error');
        navigateToHub();
        return;
    }
    
    console.log('Found humidor:', humidor);
    
    // Set current humidor permission (default to 'full' if owner or not specified)
    currentHumidorPermission = humidor.permission_level || 'full';
    console.log('Current humidor permission:', currentHumidorPermission);
    
    const welcomeSection = document.getElementById('welcomeSection');
    const mainContentSection = document.getElementById('mainContentSection');
    const humidorsContainer = document.getElementById('humidorsContainer');
    const paginationContainer = document.getElementById('paginationContainer');
    
    console.log('Setting display properties...');
    console.log('welcomeSection:', welcomeSection);
    console.log('mainContentSection:', mainContentSection);
    welcomeSection.style.display = 'none';
    mainContentSection.style.display = 'block';
    console.log('mainContentSection display after setting:', mainContentSection.style.display);
    console.log('mainContentSection computed display:', window.getComputedStyle(mainContentSection).display);
    
    // Show pagination on detail view
    if (paginationContainer) {
        paginationContainer.style.display = 'flex';
    }
    
    // Load cigars for this specific humidor with pagination
    try {
        console.log('Loading cigars for humidor:', humidorId);
        
        // Check if filters are active
        const hasFilters = searchQuery || selectedBrands.length > 0 || selectedSizes.length > 0 || 
                          selectedOrigins.length > 0 || selectedStrengths.length > 0 || 
                          selectedRingGauges.length > 0;
        
        let cigarsToDisplay;
        
        if (hasFilters) {
            // Use filtered cigars for this humidor
            console.log('Using filtered cigars');
            cigarsToDisplay = filteredCigars.filter(c => c.humidor_id === humidorId);
            totalCigars = cigarsToDisplay.length;
            totalCigarPages = 1; // No pagination when filtering
        } else {
            // Load fresh from API with pagination
            const response = await makeAuthenticatedRequest(
                `/api/v1/cigars?humidor_id=${humidorId}&page=${currentCigarPage}&page_size=${cigarPageSize}`
            );
            
            if (response.ok) {
                const responseData = await response.json();
                cigarsToDisplay = responseData.cigars || [];
                // Update the global cigars array with this humidor's cigars, but keep cigars from other humidors
                // Remove old cigars for this humidor from global array
                cigars = cigars.filter(c => c.humidor_id !== humidorId);
                // Add the freshly loaded cigars for this humidor
                cigars.push(...cigarsToDisplay);
                totalCigars = responseData.total || 0;
                totalCigarPages = responseData.total_pages || 1;
            } else {
                throw new Error('Failed to load humidor details');
            }
        }
        
        console.log('Displaying cigars:', cigarsToDisplay.length);
        
        // Render back button and humidor header
        const backButtonHtml = humidors.length > 1 ? `
            <div class="humidor-detail-nav">
                <button class="btn-back" onclick="navigateToHub()">
                    <i class="mdi mdi-arrow-left"></i> Back to Humidors
                </button>
            </div>
        ` : '';
        
        // Render the humidor section
        const renderedContent = `
            ${backButtonHtml}
            ${renderSingleHumidorSection(humidor, cigarsToDisplay)}
        `;
        console.log('Rendered content length:', renderedContent.length);
        console.log('First 200 chars of rendered content:', renderedContent.substring(0, 200));
        humidorsContainer.innerHTML = renderedContent;
        console.log('humidorsContainer after render:', humidorsContainer);
        console.log('humidorsContainer children count:', humidorsContainer.children.length);
        
        // Update pagination controls (hide if filtering)
        if (hasFilters) {
            paginationContainer.style.display = 'none';
        } else {
            updatePaginationControls();
        }
        
        // Update favorite icons for all displayed cigars
        updateAllFavoriteIcons();
    } catch (error) {
        console.error('Error loading humidor detail:', error);
        showToast('Failed to load humidor details', 'error');
        navigateToHub();
    }
}

function renderSingleHumidorSection(humidor, humidorCigars) {
    const totalQuantity = humidorCigars.reduce((sum, cigar) => sum + (cigar.quantity || 0), 0);
    
    return `
        <div class="humidor-section" data-humidor-id="${humidor.id}">
            <div class="humidor-header">
                <div class="humidor-info-card">
                    <div class="humidor-title-section">
                        <i class="mdi mdi-home-variant"></i>
                        <div>
                            <h2 class="humidor-name">${escapeHtml(humidor.name)}</h2>
                            ${humidor.location ? `<p class="humidor-location">${escapeHtml(humidor.location)}</p>` : ''}
                            ${currentHumidorPermission !== 'full' ? `<span class="permission-badge" style="background: #3498db; color: white; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; margin-left: 8px;">SHARED - ${currentHumidorPermission.toUpperCase()}</span>` : ''}
                        </div>
                    </div>
                    <div class="humidor-actions">
                        ${currentHumidorPermission === 'full' ? `
                            <button class="btn-icon" onclick="openShareHumidorModal('${humidor.id}', '${escapeHtml(humidor.name).replace(/'/g, "\\'")}')" title="Share Humidor">
                                <i class="mdi mdi-share-variant"></i>
                            </button>
                            <button class="btn-icon" onclick="editHumidor('${humidor.id}')" title="Edit Humidor">
                                <i class="mdi mdi-pencil"></i>
                            </button>
                            <button class="btn-icon" onclick="deleteHumidor('${humidor.id}')" title="Delete Humidor">
                                <i class="mdi mdi-delete"></i>
                            </button>
                        ` : ''}
                    </div>
                </div>
                <div class="humidor-stats">
                    <div class="stat-card">
                        <i class="mdi mdi-cigar stat-icon"></i>
                        <div>
                            <div class="stat-value">${totalQuantity}</div>
                            <div class="stat-label">Total Cigars</div>
                        </div>
                    </div>
                    <div class="stat-card">
                        <i class="mdi mdi-format-list-bulleted stat-icon"></i>
                        <div>
                            <div class="stat-value">${humidorCigars.length}</div>
                            <div class="stat-label">Cigar Types</div>
                        </div>
                    </div>
                </div>
                ${humidor.notes ? `
                    <div class="humidor-notes-card">
                        <i class="mdi mdi-note-text"></i>
                        <p>${escapeHtml(humidor.notes)}</p>
                    </div>
                ` : ''}
            </div>
            ${humidorCigars.length > 0 ? `
                <div class="cigars-grid">
                    ${humidorCigars.map(cigar => renderCigarCard(cigar)).join('')}
                </div>
            ` : `
                <div class="empty-state">
                    <i class="mdi mdi-cigar-off"></i>
                    <p>No cigars in this humidor yet</p>
                </div>
            `}
        </div>
    `;
}

// Wrapper function for cigar card rendering
function renderCigarCard(cigar) {
    return createCigarCard(cigar);
}

// Search and Filter Functions
function applySearchAndFilters() {
    // Start with all cigars
    filteredCigars = [...cigars];
    
    // Apply search query
    if (searchQuery) {
        const query = searchQuery.toLowerCase();
        filteredCigars = filteredCigars.filter(cigar => 
            (cigar.brand_name || getBrandName(cigar.brand_id))?.toLowerCase().includes(query) ||
            cigar.name?.toLowerCase().includes(query) ||
            (cigar.size_name || getSizeName(cigar.size_id))?.toLowerCase().includes(query) ||
            (cigar.origin_name || getOriginName(cigar.origin_id))?.toLowerCase().includes(query) ||
            cigar.wrapper?.toLowerCase().includes(query) ||
            cigar.notes?.toLowerCase().includes(query)
        );
    }
    
    // Apply brand filter (comparing brand names)
    if (selectedBrands.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedBrands.includes(cigar.brand_name || getBrandName(cigar.brand_id))
        );
    }
    
    // Apply size filter (comparing size names)
    if (selectedSizes.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedSizes.includes(cigar.size_name || getSizeName(cigar.size_id))
        );
    }
    
    // Apply origin filter (comparing origin names)
    if (selectedOrigins.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedOrigins.includes(cigar.origin_name || getOriginName(cigar.origin_id))
        );
    }
    
    // Apply strength filter (comparing strength names)
    if (selectedStrengths.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedStrengths.includes(cigar.strength_name || getStrengthName(cigar.strength_id))
        );
    }
    
    // Apply ring gauge filter (comparing ring gauge values as strings)
    if (selectedRingGauges.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedRingGauges.includes(String(cigar.ring_gauge) || getRingGaugeName(cigar.ring_gauge_id))
        );
    }
    
    updateFilterBadges();
    
    // Render based on current view - stay in detail view if we're there, otherwise show hub
    if (currentRoute.view === 'detail' && currentRoute.humidorId) {
        // Re-render the same detail view with filtered cigars
        showHumidorDetail(currentRoute.humidorId);
    } else {
        // Hub view - show all humidors with filtered cigars
        renderHumidorSections();
    }
}

function updateFilterBadges() {
    document.getElementById('brandBadge').textContent = selectedBrands.length;
    document.getElementById('sizeBadge').textContent = selectedSizes.length;
    document.getElementById('originBadge').textContent = selectedOrigins.length;
    document.getElementById('strengthBadge').textContent = selectedStrengths.length;
    document.getElementById('ringGaugeBadge').textContent = selectedRingGauges.length;
    
    // Update chip active states
    document.getElementById('filterBrandChip').setAttribute('data-active', selectedBrands.length > 0);
    document.getElementById('filterSizeChip').setAttribute('data-active', selectedSizes.length > 0);
    document.getElementById('filterOriginChip').setAttribute('data-active', selectedOrigins.length > 0);
    document.getElementById('filterStrengthChip').setAttribute('data-active', selectedStrengths.length > 0);
    document.getElementById('filterRingGaugeChip').setAttribute('data-active', selectedRingGauges.length > 0);
    
    // Show/hide clear filters button
    const hasFilters = selectedBrands.length > 0 || selectedSizes.length > 0 || 
                      selectedOrigins.length > 0 || selectedStrengths.length > 0 || 
                      selectedRingGauges.length > 0 || searchQuery;
    document.getElementById('clearFiltersBtn').style.display = hasFilters ? 'inline-flex' : 'none';
    
    // Update mobile filter toggle badge
    updateFilterToggleBadge();
}

function clearFilters() {
    searchQuery = '';
    selectedBrands = [];
    selectedSizes = [];
    selectedOrigins = [];
    selectedStrengths = [];
    selectedRingGauges = [];
    document.getElementById('cigarSearchInput').value = '';
    
    // Clear filters and stay in current view
    updateFilterBadges();
    
    if (currentRoute.view === 'detail' && currentRoute.humidorId) {
        // Stay in detail view, just clear the filters
        showHumidorDetail(currentRoute.humidorId);
    } else {
        // In hub view, clear filters and show hub (cards view if multiple humidors)
        if (humidors.length > 1) {
            showHumidorHub();
        } else {
            applySearchAndFilters();
        }
    }
}

// Filter Modal Functions
let currentFilterType = '';
let currentFilterItems = [];
let tempSelectedItems = [];

function openFilterModal(filterType) {
    currentFilterType = filterType;
    const modal = document.getElementById('filterModal');
    const title = document.getElementById('filterModalTitle');
    const filterList = document.getElementById('filterList');
    const searchInput = document.getElementById('filterSearchInput');
    
    // Set title based on filter type
    const titles = {
        'brand': 'Select Brands',
        'size': 'Select Sizes',
        'origin': 'Select Origins',
        'strength': 'Select Strengths',
        'ringGauge': 'Select Ring Gauges'
    };
    title.textContent = titles[filterType] || 'Select Filters';
    
    // Get values from the organizer arrays instead of extracting from cigars
    let uniqueValues = [];
    switch(filterType) {
        case 'brand':
            uniqueValues = brands.map(b => b.name).sort();
            tempSelectedItems = [...selectedBrands];
            break;
        case 'size':
            uniqueValues = sizes.map(s => s.name).sort();
            tempSelectedItems = [...selectedSizes];
            break;
        case 'origin':
            uniqueValues = origins.map(o => o.name).sort();
            tempSelectedItems = [...selectedOrigins];
            break;
        case 'strength':
            uniqueValues = strengths.map(s => s.name).sort();
            tempSelectedItems = [...selectedStrengths];
            break;
        case 'ringGauge':
            uniqueValues = ringGauges.map(rg => rg.gauge.toString()).sort((a, b) => parseInt(a) - parseInt(b));
            tempSelectedItems = [...selectedRingGauges];
            break;
    }
    
    currentFilterItems = uniqueValues;
    renderFilterList(uniqueValues);
    
    searchInput.value = '';
    modal.style.display = 'flex';
}

function closeFilterModal() {
    document.getElementById('filterModal').style.display = 'none';
    currentFilterType = '';
    currentFilterItems = [];
    tempSelectedItems = [];
}

function renderFilterList(items) {
    const filterList = document.getElementById('filterList');
    
    if (items.length === 0) {
        filterList.innerHTML = '<div style="padding: 1rem; text-align: center; color: var(--text-muted);">No items found</div>';
        return;
    }
    
    filterList.innerHTML = items.map(item => `
        <div class="filter-item">
            <input type="checkbox" id="filter-${item}" value="${item}" ${tempSelectedItems.includes(item) ? 'checked' : ''}>
            <label for="filter-${item}">${item}</label>
        </div>
    `).join('');
    
    // Add event listeners to checkboxes
    filterList.querySelectorAll('input[type="checkbox"]').forEach(checkbox => {
        checkbox.addEventListener('change', (e) => {
            const value = e.target.value;
            if (e.target.checked) {
                if (!tempSelectedItems.includes(value)) {
                    tempSelectedItems.push(value);
                }
            } else {
                tempSelectedItems = tempSelectedItems.filter(item => item !== value);
            }
        });
    });
}

function applyFilterSelection() {
    switch(currentFilterType) {
        case 'brand':
            selectedBrands = [...tempSelectedItems];
            break;
        case 'size':
            selectedSizes = [...tempSelectedItems];
            break;
        case 'origin':
            selectedOrigins = [...tempSelectedItems];
            break;
        case 'strength':
            selectedStrengths = [...tempSelectedItems];
            break;
        case 'ringGauge':
            selectedRingGauges = [...tempSelectedItems];
            break;
    }
    
    closeFilterModal();
    applySearchAndFilters();
}

function clearFilterSelection() {
    tempSelectedItems = [];
    renderFilterList(currentFilterItems);
}

function filterSearchHandler(query) {
    const filtered = currentFilterItems.filter(item => 
        item.toLowerCase().includes(query.toLowerCase())
    );
    renderFilterList(filtered);
}

function renderHumidorSections() {
    console.log('=== renderHumidorSections() called ===');
    const container = document.getElementById('humidorsContainer');
    
    if (humidors.length === 0) {
        console.log('✗ No humidors to render');
        container.innerHTML = '';
        return;
    }
    
    // Use filtered cigars if any filters are active, otherwise use all cigars
    const cigarsToDisplay = (searchQuery || selectedBrands.length > 0 || selectedSizes.length > 0 || 
                            selectedOrigins.length > 0 || selectedStrengths.length > 0 || 
                            selectedRingGauges.length > 0) ? filteredCigars : cigars;
    
    console.log(`→ Total cigars available: ${cigars.length}`);
    console.log(`→ Cigars to display (after filters): ${cigarsToDisplay.length}`);
    
    container.innerHTML = humidors.map(humidor => {
        const humidorCigars = cigarsToDisplay.filter(cigar => {
            const matches = cigar.humidor_id === humidor.id;
            console.log(`  Comparing cigar.humidor_id="${cigar.humidor_id}" (${typeof cigar.humidor_id}) with humidor.id="${humidor.id}" (${typeof humidor.id}) = ${matches}`);
            return matches;
        }).sort((a, b) => {
            // Sort by brand name alphabetically
            const brandA = getBrandName(a.brand_id).toLowerCase();
            const brandB = getBrandName(b.brand_id).toLowerCase();
            if (brandA < brandB) return -1;
            if (brandA > brandB) return 1;
            // If brands are the same, sort by cigar name
            return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
        });
        console.log(`→ Rendering humidor "${humidor.name}" (${humidor.id}) with ${humidorCigars.length} cigars`, humidorCigars);
        return createHumidorSection(humidor, humidorCigars);
    }).join('');
    
    console.log('✓ Humidor sections rendered');
    
    // Update favorite icons for all displayed cigars
    updateAllFavoriteIcons();
}

async function updateAllFavoriteIcons() {
    try {
        const favorites = await FavoritesAPI.getFavorites();
        const favoriteIds = new Set(favorites.map(fav => fav.cigar_id));
        
        document.querySelectorAll('.favorite-btn').forEach(btn => {
            const cigarId = btn.getAttribute('data-cigar-id');
            const isFavorite = favoriteIds.has(cigarId);
            updateFavoriteIcon(cigarId, isFavorite);
        });
    } catch (error) {
        console.error('Error updating favorite icons:', error);
    }
}

function createHumidorSection(humidor, humidorCigars) {
    const cigarCount = humidorCigars.length;
    const capacityPercentage = humidor.capacity ? Math.round((cigarCount / humidor.capacity) * 100) : 0;
    
    return `
        <div class="humidor-section" data-humidor-id="${humidor.id}">
            <div class="humidor-section-header">
                <div class="humidor-info">
                    <h2 class="humidor-title">${humidor.name}</h2>
                    <div class="humidor-meta">
                        <span class="humidor-type">${humidor.type || 'Humidor'}</span>
                        <span class="humidor-count">${cigarCount}/${humidor.capacity || '∞'} cigars</span>
                        <span class="humidor-capacity">${capacityPercentage}% full</span>
                    </div>
                </div>
                <div class="humidor-actions">
                    <button class="action-btn edit-btn" onclick="editHumidor('${humidor.id}')" title="Edit Humidor">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteHumidor('${humidor.id}')" title="Delete Humidor">🗑️</button>
                </div>
            </div>
            
            <div class="cigars-grid" data-humidor-id="${humidor.id}">
                ${humidorCigars.length > 0 
                    ? humidorCigars.map(cigar => createCigarCard(cigar)).join('') 
                    : '<div class="empty-cigars-message">No cigars in this humidor yet.</div>'
                }
            </div>
        </div>
    `;
}

function createCigarCard(cigar) {
    // Use brand_name from API response (for shared humidors), fallback to lookup
    const brandName = cigar.brand_name || getBrandName(cigar.brand_id);
    const isOutOfStock = !cigar.is_active;
    
    console.log(`→ Creating card for "${cigar.name}" with image_url:`, cigar.image_url, 'is_active:', cigar.is_active);
    
    // Determine image source or use placeholder
    const imageHtml = cigar.image_url 
        ? `<img src="${cigar.image_url}" alt="${cigar.name}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
           <img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="display: none; width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`
        : `<img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`;
    
    // Different styling and functionality for out of stock cigars
    const cardStyle = isOutOfStock ? 'style="opacity: 0.7;"' : '';
    const outOfStockBadge = isOutOfStock ? '<div class="out-of-stock-badge" style="position: absolute; top: 10px; left: 50%; transform: translateX(-50%); background: #e74c3c; color: white; padding: 4px 12px; border-radius: 4px; font-size: 0.875rem; font-weight: 600; z-index: 1; white-space: nowrap;">OUT OF STOCK</div>' : '';
    
    // For out of stock cigars, show restock button instead of delete, disable quantity controls
    const actionButtons = isOutOfStock 
        ? `<button class="action-btn edit-btn" onclick="restockCigar('${cigar.id}')" title="Restock" style="background: #27ae60;">↻</button>`
        : `<button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')" title="Edit">✏️</button>
           <button class="action-btn delete-btn" onclick="deleteCigar('${cigar.id}')" title="Mark as out of stock">🗑️</button>`;
    
    const quantityControls = isOutOfStock
        ? `<div class="cigar-card-quantity" style="opacity: 0.5;">
               <span class="quantity-value" style="color: #e74c3c;">0</span>
           </div>`
        : `<div class="cigar-card-quantity" onclick="event.stopPropagation();">
               <button class="quantity-btn" onclick="updateCigarQuantity('${cigar.id}', ${cigar.quantity}, -1)" title="Decrease quantity">−</button>
               <span class="quantity-value">${cigar.quantity}</span>
               <button class="quantity-btn" onclick="updateCigarQuantity('${cigar.id}', ${cigar.quantity}, 1)" title="Increase quantity">+</button>
           </div>`;
    
    return `
        <div class="cigar-card" data-cigar-id="${cigar.id}" onclick="openReportCard('${cigar.id}')" ${cardStyle}>
            <div class="cigar-card-image">
                ${outOfStockBadge}
                ${imageHtml}
                <button class="favorite-btn" data-cigar-id="${cigar.id}" onclick="event.stopPropagation(); toggleFavorite('${cigar.id}')" title="Add to favorites">
                    <span class="favorite-icon">♡</span>
                </button>
                <div class="cigar-card-actions" onclick="event.stopPropagation();">
                    ${actionButtons}
                </div>
            </div>
            <div class="cigar-card-content">
                <div class="cigar-card-brand">${brandName}</div>
                <h3 class="cigar-card-name">${cigar.name}</h3>
                <div class="cigar-card-footer">
                    ${getStrengthIndicatorHtml(cigar.strength_id)}
                    ${quantityControls}
                </div>
            </div>
        </div>
    `;
}

function openHumidorModal(humidor = null) {
    isEditingHumidor = !!humidor;
    currentHumidor = humidor;
    
    const modal = document.getElementById('humidorModal');
    const title = document.getElementById('humidorModalTitle');
    const saveBtn = document.getElementById('saveHumidorBtn');
    const form = document.getElementById('humidorForm');
    
    title.textContent = isEditingHumidor ? 'Edit Humidor' : 'Add New Humidor';
    saveBtn.textContent = isEditingHumidor ? 'Save Changes' : 'Create Humidor';
    
    if (isEditingHumidor) {
        // Populate form with humidor data
        Object.keys(humidor).forEach(key => {
            const input = document.getElementById(`humidor${key.charAt(0).toUpperCase() + key.slice(1)}`);
            if (input) input.value = humidor[key] || '';
        });
    } else {
        form.reset();
    }
    
    modal.classList.add('show');
}

function closeHumidorModal() {
    const modal = document.getElementById('humidorModal');
    modal.classList.remove('show');
    isEditingHumidor = false;
    currentHumidor = null;
}

// Report Card Modal Functions
function openReportCard(cigarId) {
    // Check both regular cigars and wish list cigars
    let cigar = cigars.find(c => c.id === cigarId);
    if (!cigar) {
        cigar = wishListCigars.find(c => c.id === cigarId);
    }
    if (!cigar) {
        console.error('Cigar not found:', cigarId);
        return;
    }
    
    const modal = document.getElementById('reportCardModal');
    
    // Set image
    const image = document.getElementById('reportCardImage');
    image.src = cigar.image_url || '/static/cigar-placeholder.png';
    
    // Set brand and name
    document.getElementById('reportCardBrand').textContent = cigar.brand_name || getBrandName(cigar.brand_id);
    document.getElementById('reportCardName').textContent = cigar.name;
    
    // Set details
    const humidor = humidors.find(h => h.id === cigar.humidor_id);
    document.getElementById('reportCardHumidor').textContent = humidor ? humidor.name : '-';
    document.getElementById('reportCardQuantity').textContent = cigar.quantity || '-';
    document.getElementById('reportCardSize').textContent = cigar.size_name || getSizeName(cigar.size_id);
    document.getElementById('reportCardRingGauge').textContent = cigar.ring_gauge || getRingGaugeName(cigar.ring_gauge_id);
    document.getElementById('reportCardStrength').textContent = cigar.strength_name || getStrengthName(cigar.strength_id);
    document.getElementById('reportCardOrigin').textContent = cigar.origin_name || getOriginName(cigar.origin_id);
    document.getElementById('reportCardPrice').textContent = cigar.price ? `$${parseFloat(cigar.price).toFixed(2)}` : '-';
    
    // Format purchase date
    if (cigar.purchase_date) {
        const date = new Date(cigar.purchase_date);
        document.getElementById('reportCardPurchaseDate').textContent = date.toLocaleDateString();
    } else {
        document.getElementById('reportCardPurchaseDate').textContent = '-';
    }
    
    document.getElementById('reportCardNotes').textContent = cigar.notes || 'No notes available';
    
    // Set retail link
    const retailLinkSection = document.getElementById('reportCardRetailLinkField');
    const retailLinkContainer = document.getElementById('reportCardRetailLink');
    if (cigar.retail_link) {
        retailLinkSection.style.display = 'block';
        retailLinkContainer.innerHTML = `<a href="${cigar.retail_link}" target="_blank" rel="noopener noreferrer">${cigar.retail_link}</a>`;
    } else {
        retailLinkSection.style.display = 'none';
    }
    
    // Check if this cigar is in the wish list (cigars in wish list don't have humidor_id)
    const isInWishList = !cigar.humidor_id || cigar.humidor_id === null;
    
    // Set up action buttons
    const actionsContainer = document.querySelector('.report-card-actions');
    const editBtn = document.getElementById('reportCardEditBtn');
    const deleteBtn = document.getElementById('reportCardDeleteBtn');
    
    // Remove any existing "Move to Humidor" button
    const existingMoveBtn = document.getElementById('reportCardMoveBtn');
    if (existingMoveBtn) {
        existingMoveBtn.remove();
    }
    
    // Add "Move to Humidor" button if in wish list
    if (isInWishList) {
        const moveBtn = document.createElement('button');
        moveBtn.id = 'reportCardMoveBtn';
        moveBtn.className = 'btn-primary';
        moveBtn.innerHTML = '<span class="mdi mdi-treasure-chest-outline"></span> MOVE TO HUMIDOR';
        moveBtn.onclick = async () => {
            closeReportCard();
            await moveCigarToHumidor(cigarId);
        };
        // Insert before the edit button
        actionsContainer.insertBefore(moveBtn, editBtn);
    }
    
    // Show/hide transfer button based on whether cigar is in a humidor
    const transferBtn = document.getElementById('reportCardTransferBtn');
    if (!isInWishList && humidors.length > 1) {
        // Show transfer button only if cigar is in a humidor and there are other humidors to transfer to
        transferBtn.style.display = 'block';
        transferBtn.onclick = () => {
            openTransferModal(cigar);
        };
    } else {
        transferBtn.style.display = 'none';
    }
    
    editBtn.onclick = () => {
        closeReportCard();
        editCigar(cigarId);
    };
    
    deleteBtn.onclick = async () => {
        closeReportCard();
        await deleteCigar(cigarId);
    };
    
    modal.classList.add('show');
}

function closeReportCard() {
    const modal = document.getElementById('reportCardModal');
    modal.classList.remove('show');
}

// Transfer Modal Functions
let currentTransferCigar = null;

function openTransferModal(cigar) {
    currentTransferCigar = cigar;
    const modal = document.getElementById('transferCigarModal');
    
    // Set cigar name
    document.getElementById('transferCigarName').value = `${cigar.brand_name || getBrandName(cigar.brand_id)} - ${cigar.name}`;
    
    // Set source humidor
    const sourceHumidor = humidors.find(h => h.id === cigar.humidor_id);
    document.getElementById('transferSourceHumidor').value = sourceHumidor ? sourceHumidor.name : '-';
    
    // Populate destination humidor dropdown (exclude current humidor)
    const destinationSelect = document.getElementById('transferDestinationHumidor');
    destinationSelect.innerHTML = '<option value="">-- Select destination humidor --</option>';
    
    humidors.forEach(humidor => {
        if (humidor.id !== cigar.humidor_id) {
            const option = document.createElement('option');
            option.value = humidor.id;
            option.textContent = humidor.name;
            destinationSelect.appendChild(option);
        }
    });
    
    // Set quantity info
    const quantityInput = document.getElementById('transferQuantity');
    quantityInput.max = cigar.quantity;
    quantityInput.value = 1;
    document.getElementById('transferAvailable').textContent = `(Available: ${cigar.quantity})`;
    
    // Attach event handlers directly here to ensure they work
    const closeBtn = document.getElementById('closeTransferModal');
    const cancelBtn = document.getElementById('cancelTransferBtn');
    const confirmBtn = document.getElementById('confirmTransferBtn');
    const transferAllBtn = document.getElementById('transferAllBtn');
    
    if (closeBtn) {
        closeBtn.onclick = (e) => {
            e.preventDefault();
            closeTransferModal();
        };
    }
    
    if (cancelBtn) {
        cancelBtn.onclick = (e) => {
            e.preventDefault();
            closeTransferModal();
        };
    }
    
    if (confirmBtn) {
        confirmBtn.onclick = (e) => {
            e.preventDefault();
            performTransfer();
        };
    }
    
    if (transferAllBtn) {
        transferAllBtn.onclick = (e) => {
            e.preventDefault();
            if (currentTransferCigar) {
                document.getElementById('transferQuantity').value = currentTransferCigar.quantity;
            }
        };
    }
    
    modal.classList.add('show');
}

function closeTransferModal() {
    const modal = document.getElementById('transferCigarModal');
    if (modal) {
        modal.classList.remove('show');
    }
    currentTransferCigar = null;
}

async function performTransfer() {
    if (!currentTransferCigar) return;
    
    const destinationHumidorId = document.getElementById('transferDestinationHumidor').value;
    const quantity = parseInt(document.getElementById('transferQuantity').value);
    
    if (!destinationHumidorId) {
        showToast('Please select a destination humidor', 'error');
        return;
    }
    
    if (!quantity || quantity < 1 || quantity > currentTransferCigar.quantity) {
        showToast(`Please enter a valid quantity (1-${currentTransferCigar.quantity})`, 'error');
        return;
    }
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${currentTransferCigar.id}/transfer`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                destination_humidor_id: destinationHumidorId,
                quantity: quantity
            })
        });
        
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.message || 'Failed to transfer cigar');
        }
        
        showToast('Cigar transferred successfully!', 'success');
        closeTransferModal();
        closeReportCard();
        await loadHumidors();
        // Force re-render of the current view to update stats
        showHumidorHub();
        
    } catch (error) {
        console.error('Error transferring cigar:', error);
        showToast(error.message || 'Failed to transfer cigar', 'error');
    }
}

function openCigarModal(humidorId = null, cigar = null) {
    console.log('=== openCigarModal() called ===');
    console.log('→ humidorId:', humidorId);
    console.log('→ cigar:', cigar);
    console.log('→ Available humidors:', humidors.length);
    
    isEditingCigar = !!cigar;
    currentCigar = cigar;
    
    const modal = document.getElementById('cigarModal');
    const title = document.getElementById('cigarModalTitle');
    const form = document.getElementById('cigarForm');
    const humidorSelect = document.getElementById('cigarHumidor');
    
    if (!humidorSelect) {
        console.error('✗ cigarHumidor select element not found!');
        return;
    }
    
    title.textContent = isEditingCigar ? 'Edit Cigar' : 'Add New Cigar';
    
    // Populate humidor dropdown
    humidorSelect.innerHTML = '<option value="">Select Humidor</option>';
    
    // Add all humidors
    humidors.forEach(humidor => {
        const option = document.createElement('option');
        option.value = humidor.id;
        option.textContent = humidor.name;
        if (humidorId && humidor.id === humidorId) {
            option.selected = true;
        }
        humidorSelect.appendChild(option);
    });
    
    // Always add Wish List option last (special identifier)
    const wishListOpt = document.createElement('option');
    wishListOpt.value = 'WISH_LIST';
    wishListOpt.textContent = '📝 Wish List';
    if (humidorId === 'WISH_LIST') {
        wishListOpt.selected = true;
    }
    humidorSelect.appendChild(wishListOpt);
    
    console.log(`✓ Humidor dropdown populated with ${humidors.length} options`);
    
    // Populate organizer dropdowns
    populateOrganizerDropdowns();
    
    if (isEditingCigar) {
        // Populate form with cigar data
        form.reset();  // Start fresh
        
        // Set direct fields
        document.getElementById('cigarName').value = cigar.name || '';
        document.getElementById('cigarQuantity').value = cigar.quantity || 1;
        document.getElementById('cigarPrice').value = cigar.price || '';
        document.getElementById('cigarRetailLink').value = cigar.retail_link || '';
        document.getElementById('cigarNotes').value = cigar.notes || '';
        document.getElementById('cigarImageUrl').value = cigar.image_url || '';
        
        if (cigar.purchase_date) {
            document.getElementById('cigarPurchaseDate').value = cigar.purchase_date.split('T')[0];
        }
        
        // Set organizer dropdowns using IDs
        if (cigar.humidor_id) {
            humidorSelect.value = cigar.humidor_id;
        } else {
            // If humidor_id is null, this is a wish list cigar
            humidorSelect.value = 'WISH_LIST';
        }
        if (cigar.brand_id) document.getElementById('cigarBrand').value = cigar.brand_id;
        if (cigar.size_id) document.getElementById('cigarSize').value = cigar.size_id;
        if (cigar.origin_id) document.getElementById('cigarOrigin').value = cigar.origin_id;
        if (cigar.strength_id) document.getElementById('cigarStrength').value = cigar.strength_id;
        if (cigar.ring_gauge_id) document.getElementById('cigarRingGauge').value = cigar.ring_gauge_id;
    } else {
        form.reset();
        if (humidorId) {
            humidorSelect.value = humidorId;
        }
    }
    
    modal.classList.add('show');
}

function populateOrganizerDropdowns() {
    // Populate Brand dropdown
    const brandSelect = document.getElementById('cigarBrand');
    if (brandSelect) {
        brandSelect.innerHTML = '<option value="">Select or type brand</option>';
        brands.forEach(brand => {
            const option = document.createElement('option');
            option.value = brand.id;  // Use ID instead of name
            option.textContent = brand.name;
            brandSelect.appendChild(option);
        });
    }
    
    // Populate Size dropdown
    const sizeSelect = document.getElementById('cigarSize');
    if (sizeSelect) {
        sizeSelect.innerHTML = '<option value="">Select size</option>';
        sizes.forEach(size => {
            const option = document.createElement('option');
            option.value = size.id;  // Use ID instead of name
            option.textContent = size.name;
            sizeSelect.appendChild(option);
        });
    }
    
    // Populate Strength dropdown
    const strengthSelect = document.getElementById('cigarStrength');
    if (strengthSelect) {
        strengthSelect.innerHTML = '<option value="">Select strength</option>';
        strengths.forEach(strength => {
            const option = document.createElement('option');
            option.value = strength.id;  // Use ID instead of name
            option.textContent = strength.name;
            strengthSelect.appendChild(option);
        });
    }
    
    // Populate Origin dropdown
    const originSelect = document.getElementById('cigarOrigin');
    if (originSelect) {
        originSelect.innerHTML = '<option value="">Select origin</option>';
        origins.forEach(origin => {
            const option = document.createElement('option');
            option.value = origin.id;  // Use ID instead of name
            option.textContent = origin.name;
            originSelect.appendChild(option);
        });
    }
    
    // Populate Ring Gauge dropdown
    const ringGaugeSelect = document.getElementById('cigarRingGauge');
    if (ringGaugeSelect) {
        ringGaugeSelect.innerHTML = '<option value="">Select ring gauge</option>';
        ringGauges.forEach(rg => {
            const option = document.createElement('option');
            option.value = rg.id;  // Use ID instead of gauge number
            option.textContent = `${rg.gauge}${rg.common_names && rg.common_names.length > 0 ? ' (' + rg.common_names.join(', ') + ')' : ''}`;
            ringGaugeSelect.appendChild(option);
        });
    }
}

function closeCigarModal() {
    const modal = document.getElementById('cigarModal');
    modal.classList.remove('show');
    isEditingCigar = false;
    currentCigar = null;
    
    // Reset file upload display
    const uploadFileName = document.getElementById('uploadFileName');
    if (uploadFileName) {
        uploadFileName.textContent = 'JPG or PNG, max 5MB';
    }
}

async function saveHumidor() {
    const form = document.getElementById('humidorForm');
    const formData = new FormData(form);
    
    // Check if there's an image file to upload (same pattern as cigars)
    const imageFile = document.getElementById('humidorImageUpload').files[0];
    let imageUrl = formData.get('image_url') || null;
    
    // If a file is selected, convert to base64 data URL
    if (imageFile) {
        // Validate file size (5MB limit, same as cigars)
        if (imageFile.size > 5 * 1024 * 1024) {
            showToast('Image must be under 5MB', 'error');
            return;
        }
        
        try {
            imageUrl = await new Promise((resolve, reject) => {
                const reader = new FileReader();
                reader.onload = (e) => resolve(e.target.result);
                reader.onerror = reject;
                reader.readAsDataURL(imageFile);
            });
        } catch (error) {
            console.error('Error reading image file:', error);
            showToast('Failed to process image', 'error');
            return;
        }
    }
    
    const humidorData = {
        name: formData.get('name'),
        type: formData.get('type') || null,
        capacity: parseInt(formData.get('capacity')),
        location: formData.get('location') || null,
        description: formData.get('description') || null,
        image_url: imageUrl
    };

    try {
        if (isEditingHumidor && currentHumidor) {
            // Update existing humidor
            const response = await makeAuthenticatedRequest(`/api/v1/humidors/${currentHumidor.id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(humidorData)
            });

            if (response.ok) {
                await loadHumidors();
                showToast('Humidor updated successfully!', 'success');
                closeHumidorModal();
            } else {
                const errorData = await response.json();
                showToast(errorData.error || 'Failed to update humidor', 'error');
            }
        } else {
            // Create new humidor
            const response = await makeAuthenticatedRequest('/api/v1/humidors', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(humidorData)
            });

            if (response.ok) {
                await loadHumidors();
                showToast('Humidor created successfully!', 'success');
                closeHumidorModal();
            } else {
                const errorData = await response.json();
                showToast(errorData.error || 'Failed to create humidor', 'error');
            }
        }
    } catch (error) {
        console.error('Error saving humidor:', error);
        
        // Check for specific error types
        if (error.message === 'PAYLOAD_TOO_LARGE') {
            showToast('Image is too large. Please use a smaller image (under 5MB).', 'error');
        } else if (error.message === 'Failed to fetch' || error.name === 'TypeError') {
            // Network error or connection reset - likely payload size issue
            showToast('Request failed - image may be too large. Try a smaller image.', 'error');
        } else {
            showToast('Failed to save humidor', 'error');
        }
    }
}

function createHumidor(humidorData) {
    const newHumidor = {
        id: Date.now().toString(),
        ...humidorData,
        created_at: new Date().toISOString()
    };
    
    humidors.push(newHumidor);
    showAppropriateSection();
    showToast('Humidor created successfully!');
    closeHumidorModal();
}

function editHumidor(id) {
    const humidor = humidors.find(h => h.id === id);
    if (humidor) openHumidorModal(humidor);
}

function updateHumidor(id, humidorData) {
    const index = humidors.findIndex(h => h.id === id);
    if (index !== -1) {
        humidors[index] = { 
            ...humidors[index], 
            ...humidorData,
            updated_at: new Date().toISOString()
        };
        renderHumidorSections();
        showToast('Humidor updated successfully!');
        closeHumidorModal();
    }
}

async function deleteHumidor(id) {
    if (!confirm('Are you sure you want to delete this humidor and all its cigars?')) return;
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/humidors/${id}`, {
            method: 'DELETE'
        });
        
        if (response) {
            // Remove cigars from this humidor locally
            cigars = cigars.filter(cigar => cigar.humidor_id !== id);
            
            // Remove the humidor locally
            humidors = humidors.filter(h => h.id !== id);
            
            showAppropriateSection();
            showToast('Humidor deleted successfully!');
        } else {
            throw new Error('Failed to delete humidor');
        }
    } catch (error) {
        console.error('Error deleting humidor:', error);
        showToast('Failed to delete humidor', 'error');
    }
}

async function saveCigar() {
    const form = document.getElementById('cigarForm');
    const formData = new FormData(form);
    
    // Get image URL from input or handle file upload
    let imageUrl = formData.get('image_url') || null;
    const imageFile = document.getElementById('cigarImageUpload').files[0];
    
    console.log('→ Image URL from form:', imageUrl);
    console.log('→ Image file selected:', imageFile ? imageFile.name : 'none');
    
    // If a file was uploaded, convert it to base64
    if (imageFile) {
        // Check file size (max 5MB)
        const maxSize = 5 * 1024 * 1024; // 5MB in bytes
        if (imageFile.size > maxSize) {
            showToast('Image file is too large. Maximum size is 5MB.', 'error');
            return;
        }
        
        try {
            imageUrl = await new Promise((resolve, reject) => {
                const reader = new FileReader();
                reader.onload = (e) => resolve(e.target.result);
                reader.onerror = reject;
                reader.readAsDataURL(imageFile);
            });
            console.log('✓ Image file converted to base64, length:', imageUrl.length);
        } catch (error) {
            console.error('✗ Failed to read image file:', error);
            showToast('Failed to read image file', 'error');
            return;
        }
    }
    
    const selectedHumidorId = formData.get('humidor_id') || null;
    const isWishList = selectedHumidorId === 'WISH_LIST';
    
    const cigarData = {
        humidor_id: isWishList ? null : selectedHumidorId,
        brand_id: document.getElementById('cigarBrand').value || null,
        name: formData.get('name'),
        size_id: document.getElementById('cigarSize').value || null,
        origin_id: document.getElementById('cigarOrigin').value || null,
        strength_id: document.getElementById('cigarStrength').value || null,
        ring_gauge_id: document.getElementById('cigarRingGauge').value || null,
        length: formData.get('length') ? parseFloat(formData.get('length')) : null,
        quantity: parseInt(formData.get('quantity')) || 1,
        purchase_date: formData.get('purchase_date') ? new Date(formData.get('purchase_date')).toISOString() : null,
        price: formData.get('price') ? parseFloat(formData.get('price')) : null,
        retail_link: formData.get('retail_link') || null,
        notes: formData.get('notes') || null,
        image_url: imageUrl
    };

    console.log('=== saveCigar() called ===');
    console.log('→ Form data extracted:', cigarData);
    console.log('→ Is Wish List:', isWishList);
    
    // Validate that a humidor/location is selected
    if (!isWishList && (!cigarData.humidor_id || cigarData.humidor_id.trim() === '')) {
        console.error('✗ No humidor selected!');
        showToast('Please select a humidor or wish list for this cigar', 'error');
        return;
    }
    
    console.log(`✓ Location selected: ${isWishList ? 'Wish List' : cigarData.humidor_id}`);

    try {
        let response;
        if (isEditingCigar && currentCigar) {
            // Update existing cigar
            console.log(`→ Updating existing cigar ${currentCigar.id}...`);
            response = await makeAuthenticatedRequest(`/api/v1/cigars/${currentCigar.id}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(cigarData)
            });
        } else {
            // Create new cigar
            console.log('→ Creating new cigar...');
            response = await makeAuthenticatedRequest('/api/v1/cigars', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(cigarData)
            });
        }

        console.log(`→ API response status: ${response.status}`);
        
        if (response.ok) {
            const savedCigar = await response.json();
            console.log('✓ Cigar saved successfully:', savedCigar);
            
            // If this was meant for wish list, add it there
            if (isWishList) {
                console.log('→ Adding cigar to wish list...');
                console.log('  Cigar ID:', savedCigar.id);
                const wishListResponse = await makeAuthenticatedRequest('/api/v1/wish_list', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        cigar_id: savedCigar.id,
                        notes: null
                    })
                });
                
                console.log('  Wish list API response status:', wishListResponse.status);
                
                if (!wishListResponse.ok) {
                    const errorText = await wishListResponse.text();
                    console.error('✗ Failed to add to wish list:', errorText);
                    let errorData;
                    try {
                        errorData = JSON.parse(errorText);
                    } catch (e) {
                        errorData = { error: errorText };
                    }
                    console.error('  Error details:', errorData);
                    showToast(errorData.error || 'Failed to add to wish list', 'error');
                    return;
                }
                
                const wishListData = await wishListResponse.json();
                console.log('✓ Cigar added to wish list successfully:', wishListData);
            }
            
            console.log('→ Reloading data...');
            
            // Clear the file input
            document.getElementById('cigarImageUpload').value = '';
            
            // Reload data based on what's currently visible
            if (isWishList || (isEditingCigar && currentCigar && !currentCigar.humidor_id)) {
                // Refresh wish list if we're working with a wish list cigar
                console.log('→ Reloading wish list...');
                await loadWishList();
                // Also reload cigars if that view is visible
                const cigarSection = document.getElementById('cigarManagementSection');
                if (cigarSection && cigarSection.style.display !== 'none') {
                    await loadCigars();
                }
            } else {
                // Reload humidors for regular cigars
                await loadHumidors();
            }
            
            showToast(isEditingCigar ? 'Cigar updated successfully!' : 'Cigar added successfully!', 'success');
            closeCigarModal();
        } else {
            const errorData = await response.json();
            console.error('✗ Failed to save cigar:', errorData);
            showToast(errorData.message || errorData.error || 'Failed to save cigar', 'error');
        }
    } catch (error) {
        console.error('✗ Error saving cigar:', error);
        showToast('Failed to save cigar', 'error');
    }
}

function createCigar(cigarData) {
    const newCigar = {
        id: Date.now().toString(),
        ...cigarData,
        created_at: new Date().toISOString()
    };
    
    cigars.push(newCigar);
    renderHumidorSections();
    showToast('Cigar added successfully!');
    closeCigarModal();
}

function updateCigar(id, cigarData) {
    const index = cigars.findIndex(c => c.id === id);
    if (index !== -1) {
        cigars[index] = { 
            ...cigars[index], 
            ...cigarData,
            updated_at: new Date().toISOString()
        };
        renderHumidorSections();
        showToast('Cigar updated successfully!');
        closeCigarModal();
    }
}

async function deleteCigar(id) {
    if (!confirm('Are you sure you want to permanently delete this cigar from your humidor? It will remain in favorites if favorited.')) return;
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'DELETE'
        });
        
        if (response && response.ok) {
            showToast('Cigar deleted from humidor');
            await loadHumidors();
        } else {
            throw new Error('Failed to delete cigar');
        }
    } catch (error) {
        console.error('Error deleting cigar:', error);
        showToast('Failed to delete cigar', 'error');
    }
}

// Restock cigar function
async function restockCigar(id) {
    // Check permission
    if (currentHumidorPermission === 'view') {
        showToast('You only have view permission for this humidor', 'error');
        return;
    }
    
    const quantity = prompt('Enter the quantity to restock:', '1');
    
    if (quantity === null) return; // User cancelled
    
    const parsedQuantity = parseInt(quantity);
    if (isNaN(parsedQuantity) || parsedQuantity < 1) {
        showToast('Please enter a valid quantity (1 or more)', 'error');
        return;
    }
    
    try {
        // Update the cigar to set is_active=true and new quantity
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'PUT',
            body: JSON.stringify({
                quantity: parsedQuantity
            })
        });
        
        if (response && response.ok) {
            // Also need to set is_active back to true - do this via a separate backend update
            // For now, we'll update quantity which should trigger the restock
            showToast(`Cigar restocked with ${parsedQuantity} units!`, 'success');
            await loadHumidors();
        } else {
            throw new Error('Failed to restock cigar');
        }
    } catch (error) {
        console.error('Error restocking cigar:', error);
        showToast('Failed to restock cigar', 'error');
    }
}

// Quantity update function
async function updateCigarQuantity(cigarId, currentQuantity, change) {
    const newQuantity = currentQuantity + change;
    
    // Don't allow quantity to go below 0
    if (newQuantity < 0) {
        showToast('Quantity cannot be negative', 'error');
        return;
    }
    
    // If quantity reaches 0, mark as out of stock by setting is_active=false
    if (newQuantity === 0) {
        try {
            // Update with quantity 0, which will set is_active=false on the backend
            const response = await makeAuthenticatedRequest(`/api/v1/cigars/${cigarId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    quantity: 0
                })
            });
            
            if (response && response.ok) {
                showToast('Cigar marked as out of stock (quantity: 0)', 'info');
                await loadHumidors();
            } else {
                throw new Error('Failed to mark cigar as out of stock');
            }
        } catch (error) {
            console.error('Error marking cigar as out of stock:', error);
            showToast('Failed to mark cigar as out of stock', 'error');
        }
        return;
    }
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${cigarId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ quantity: newQuantity })
        });
        
        if (response && response.ok) {
            console.log(`✓ Updated quantity for cigar ${cigarId} to ${newQuantity}`);
            // Reload humidors to refresh the display
            await loadHumidors();
            showToast(`Quantity updated to ${newQuantity}`, 'success');
        } else {
            throw new Error('Failed to update quantity');
        }
    } catch (error) {
        console.error('Error updating quantity:', error);
        showToast('Failed to update quantity', 'error');
    }
}

// Profile/Account Settings Functions
async function loadUserProfile() {
    try {
        const response = await makeAuthenticatedRequest('/api/v1/users/self');
        
        if (response && response.ok) {
            const user = await response.json();
            displayUserProfile(user);
        } else {
            showToast('Failed to load user profile', 'error');
        }
    } catch (error) {
        console.error('Error loading profile:', error);
        showToast('Error loading profile', 'error');
    }
}

function displayUserProfile(user) {
    // Fill form fields
    document.getElementById('profileUsername').value = user.username || '';
    document.getElementById('profileEmail').value = user.email || '';
    document.getElementById('profileFullName').value = user.full_name || '';
}

async function saveProfile() {
    const username = document.getElementById('profileUsername').value.trim();
    const email = document.getElementById('profileEmail').value.trim();
    const fullName = document.getElementById('profileFullName').value.trim();
    
    // Validation
    if (!username) {
        showToast('Username is required', 'error');
        return;
    }
    
    if (!email) {
        showToast('Email is required', 'error');
        return;
    }
    
    if (!email.includes('@')) {
        showToast('Please enter a valid email address', 'error');
        return;
    }
    
    const updateData = {
        username,
        email,
        full_name: fullName || null
    };
    
    try {
        const saveBtn = document.getElementById('saveProfileBtn');
        saveBtn.disabled = true;
        saveBtn.innerHTML = '<span class="mdi mdi-loading mdi-spin"></span> Saving...';
        
        const response = await makeAuthenticatedRequest('/api/v1/users/self', {
            method: 'PUT',
            body: JSON.stringify(updateData)
        });
        
        if (response && response.ok) {
            const updatedUser = await response.json();
            displayUserProfile(updatedUser);
            // Update user display
            initializeUserDisplay();
            showToast('Profile updated successfully!');
        } else {
            const error = await response.json();
            showToast(error.error || 'Failed to update profile', 'error');
        }
    } catch (error) {
        console.error('Error updating profile:', error);
        showToast('Error updating profile', 'error');
    } finally {
        const saveBtn = document.getElementById('saveProfileBtn');
        saveBtn.disabled = false;
        saveBtn.innerHTML = '<span class="mdi mdi-content-save"></span> Save Profile';
    }
}

async function changePassword() {
    const currentPassword = document.getElementById('currentPassword').value;
    const newPassword = document.getElementById('newPassword').value;
    const confirmPassword = document.getElementById('confirmPassword').value;
    
    // Validation
    if (!currentPassword) {
        showToast('Current password is required', 'error');
        return;
    }
    
    if (!newPassword) {
        showToast('New password is required', 'error');
        return;
    }
    
    if (newPassword.length < 8) {
        showToast('New password must be at least 8 characters', 'error');
        return;
    }
    
    if (newPassword !== confirmPassword) {
        showToast('New passwords do not match', 'error');
        return;
    }
    
    const passwordData = {
        current_password: currentPassword,
        new_password: newPassword
    };
    
    try {
        const changeBtn = document.getElementById('changePasswordBtn');
        changeBtn.disabled = true;
        changeBtn.innerHTML = '<span class="mdi mdi-loading mdi-spin"></span> Changing...';
        
        const response = await makeAuthenticatedRequest('/api/v1/users/password', {
            method: 'PUT',
            body: JSON.stringify(passwordData)
        });
        
        if (response && response.ok) {
            const result = await response.json();
            showToast(result.message || 'Password changed successfully!');
            
            // Clear password fields
            document.getElementById('currentPassword').value = '';
            document.getElementById('newPassword').value = '';
            document.getElementById('confirmPassword').value = '';
        } else {
            const error = await response.json();
            showToast(error.error || 'Failed to change password', 'error');
        }
    } catch (error) {
        console.error('Error changing password:', error);
        showToast('Error changing password', 'error');
    } finally {
        const changeBtn = document.getElementById('changePasswordBtn');
        changeBtn.disabled = false;
        changeBtn.innerHTML = '<span class="mdi mdi-lock-check"></span> Change Password';
    }
}

// Favorites Functions
async function toggleFavorite(cigarId) {
    try {
        const status = await FavoritesAPI.isFavorite(cigarId);
        
        if (status.is_favorite) {
            await FavoritesAPI.removeFavorite(cigarId);
            showToast('Removed from favorites', 'info');
        } else {
            await FavoritesAPI.addFavorite(cigarId);
            showToast('Added to favorites!', 'success');
        }
        
        // Update heart icon on all cards with this cigar
        updateFavoriteIcon(cigarId, !status.is_favorite);
        
        // Refresh favorites page if it's currently visible
        const favoritesSection = document.getElementById('favoritesSection');
        if (favoritesSection.style.display !== 'none') {
            await loadFavorites();
        }
    } catch (error) {
        console.error('Error toggling favorite:', error);
        showToast('Failed to update favorite', 'error');
    }
}

function updateFavoriteIcon(cigarId, isFavorite) {
    const buttons = document.querySelectorAll(`.favorite-btn[data-cigar-id="${cigarId}"]`);
    buttons.forEach(btn => {
        const icon = btn.querySelector('.favorite-icon');
        if (isFavorite) {
            icon.textContent = '♥';
            btn.classList.add('is-favorite');
            btn.title = 'Remove from favorites';
        } else {
            icon.textContent = '♡';
            btn.classList.remove('is-favorite');
            btn.title = 'Add to favorites';
        }
    });
}

async function loadFavorites() {
    try {
        const favorites = await FavoritesAPI.getFavorites();
        const emptyState = document.getElementById('favoritesEmptyState');
        const favoritesGrid = document.getElementById('favoritesGrid');
        
        if (favorites.length === 0) {
            emptyState.style.display = 'block';
            favoritesGrid.style.display = 'none';
        } else {
            emptyState.style.display = 'none';
            favoritesGrid.style.display = 'grid';
            
            // Extract cigar data from nested structure
            const favoritesCigars = favorites.map(fav => ({
                ...fav.cigar,
                favorite_id: fav.id  // Keep favorite ID for removal
            }));
            
            favoritesGrid.innerHTML = favoritesCigars.map(cigar => createFavoriteCard(cigar)).join('');
            
            // Update favorite icons for all loaded favorites that still exist (not out of stock)
            favoritesCigars.filter(c => !c.out_of_stock && c.id).forEach(cigar => updateFavoriteIcon(cigar.id, true));
        }
    } catch (error) {
        console.error('Error loading favorites:', error);
        showToast('Failed to load favorites', 'error');
    }
}

function createFavoriteCard(cigar) {
    const brandName = getBrandName(cigar.brand_id);
    const humidor = humidors.find(h => h.id === cigar.humidor_id);
    const humidorName = humidor ? humidor.name : 'Unknown Humidor';
    
    const imageHtml = cigar.image_url 
        ? `<img src="${cigar.image_url}" alt="${cigar.name}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
           <img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="display: none; width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`
        : `<img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`;
    
    // Out of stock cigars should still be clickable to view report card
    const cardStyle = cigar.out_of_stock ? 'style="opacity: 0.7;"' : '';
    
    // OUT OF STOCK badge centered on card (same style as humidor cards)
    const outOfStockBadge = cigar.out_of_stock 
        ? '<div class="out-of-stock-badge" style="position: absolute; top: 10px; left: 50%; transform: translateX(-50%); background: #e74c3c; color: white; padding: 4px 12px; border-radius: 4px; font-size: 0.875rem; font-weight: 600; z-index: 10; white-space: nowrap;">OUT OF STOCK</div>'
        : '';
    
    const favoriteButton = cigar.out_of_stock 
        ? '' // No heart button for out of stock cigars
        : `<button class="favorite-btn is-favorite" data-cigar-id="${cigar.id}" onclick="event.stopPropagation(); toggleFavorite('${cigar.id}')" title="Remove from favorites">
               <span class="favorite-icon">♥</span>
           </button>`;
    
    const deleteAction = cigar.out_of_stock
        ? `onclick="removeFavorite('${cigar.favorite_id}')"` // Use favorite_id for out of stock
        : `onclick="removeFavorite('${cigar.id}')"`; // Use cigar_id for active cigars
    
    return `
        <div class="cigar-card" data-cigar-id="${cigar.id}" onclick="openReportCard('${cigar.id}')" ${cardStyle}>
            <div class="cigar-card-image" style="position: relative;">
                ${outOfStockBadge}
                ${imageHtml}
                ${favoriteButton}
                <div class="cigar-card-actions" onclick="event.stopPropagation();">
                    <button class="action-btn delete-btn" ${deleteAction} title="Remove from favorites">🗑️</button>
                </div>
            </div>
            <div class="cigar-card-content">
                <div class="cigar-card-brand">${brandName}</div>
                <h3 class="cigar-card-name">${cigar.name}</h3>
                <div class="cigar-card-humidor">
                    <i class="mdi mdi-home-variant"></i>
                    <span>${escapeHtml(humidorName)}</span>
                </div>
                ${getStrengthIndicatorHtml(cigar.strength_id)}
            </div>
        </div>
    `;
}

function getStrengthIndicatorHtml(strengthId) {
    const strengthLevel = getStrengthLevel(strengthId);
    if (!strengthLevel) return '';
    
    return `
        <div class="strength-indicator" style="margin-top: 0.5rem;" title="Strength: ${strengthLevel}/5">
            ${Array.from({length: 5}, (_, i) => 
                `<span class="strength-dot ${i < strengthLevel ? 'active' : ''}"></span>`
            ).join('')}
        </div>
    `;
}

async function removeFavorite(cigarId) {
    if (!confirm('Remove this from your favorites?')) return;
    
    try {
        await FavoritesAPI.removeFavorite(cigarId);
        showToast('Removed from favorites', 'info');
        
        // Update heart icons on all cards
        updateFavoriteIcon(cigarId, false);
        
        // Reload favorites page
        await loadFavorites();
    } catch (error) {
        console.error('Error removing favorite:', error);
        showToast('Failed to remove favorite', 'error');
    }
}

// Wish List Functions
let wishListHumidor = null;

async function loadWishList() {
    try {
        // Load wish list items using the new API
        const response = await makeAuthenticatedRequest('/api/v1/wish_list', {
            method: 'GET'
        });
        
        if (!response.ok) {
            throw new Error('Failed to fetch wish list');
        }
        
        const wishListItems = await response.json();
        console.log(`✓ Loaded ${wishListItems.length} items in wish list:`, wishListItems);
        
        // Extract cigars from wish list items and store globally
        wishListCigars = wishListItems
            .filter(item => item.cigar !== null)
            .map(item => ({
                ...item.cigar,
                wish_list_notes: item.notes,
                wish_list_id: item.id
            }));
        console.log(`✓ ${wishListCigars.length} cigars with details:`, wishListCigars);
        
        const emptyState = document.getElementById('wishlistEmptyState');
        const wishlistGrid = document.getElementById('wishlistGrid');
        
        if (wishListCigars.length === 0) {
            emptyState.style.display = 'block';
            wishlistGrid.style.display = 'none';
        } else {
            emptyState.style.display = 'none';
            wishlistGrid.style.display = 'grid';
            wishlistGrid.innerHTML = wishListCigars.map(cigar => createWishListCard(cigar)).join('');
        }
    } catch (error) {
        console.error('Error loading wish list:', error);
        showToast('Failed to load wish list', 'error');
    }
}

function createWishListCard(cigar) {
    const brandName = getBrandName(cigar.brand_id);
    
    const imageHtml = cigar.image_url 
        ? `<img src="${cigar.image_url}" alt="${cigar.name}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
           <img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="display: none; width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`
        : `<img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`;
    
    return `
        <div class="cigar-card wishlist-card" data-cigar-id="${cigar.id}" onclick="openReportCard('${cigar.id}')">
            <div class="cigar-card-image">
                ${imageHtml}
                <div class="cigar-card-actions" onclick="event.stopPropagation();">
                    <button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')" title="Edit">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteWishListCigar('${cigar.id}')" title="Delete">🗑️</button>
                </div>
            </div>
            <div class="cigar-card-content">
                <div class="cigar-card-brand">${brandName}</div>
                <h3 class="cigar-card-name">${cigar.name}</h3>
                ${getStrengthIndicatorHtml(cigar.strength_id)}
            </div>
        </div>
    `;
}

async function moveCigarToHumidor(cigarId, event) {
    event?.stopPropagation();
    
    try {
        // Get list of humidors
        const response = await makeAuthenticatedRequest('/api/v1/humidors', {
            method: 'GET'
        });
        
        if (!response.ok) {
            throw new Error('Failed to fetch humidors');
        }
        
        const allHumidors = await response.json();
        
        if (allHumidors.length === 0) {
            showToast('Please create a humidor first', 'error');
            return;
        }
        
        // Show selection modal
        const result = await showHumidorSelectionModal(allHumidors);
        
        if (!result) return; // User cancelled
        
        const { humidorId, quantity } = result;
        
        // Set purchase date to today (RFC3339 format with timezone)
        const today = new Date().toISOString();
        
        // First, update the cigar's humidor_id, quantity, and purchase date (while still in wish list for permission check)
        const updateResponse = await makeAuthenticatedRequest(`/api/v1/cigars/${cigarId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                humidor_id: humidorId,
                quantity: quantity,
                purchase_date: today
            })
        });
        
        if (!updateResponse.ok) {
            throw new Error('Failed to update cigar');
        }
        
        // Then, remove from wish list
        const removeResponse = await makeAuthenticatedRequest(`/api/v1/wish_list/${cigarId}`, {
            method: 'DELETE'
        });
        
        if (!removeResponse.ok) {
            console.warn('Failed to remove from wish list, continuing anyway');
        }
        
        showToast('Cigar moved to humidor successfully!');
        await loadHumidors(); // Reload all data
        
        // If we're on the wish list page, reload it
        if (currentPage === 'wishlist') {
            await loadWishList();
        }
    } catch (error) {
        console.error('Error moving cigar:', error);
        showToast('Failed to move cigar', 'error');
    }
}

function showHumidorSelectionModal(humidors) {
    return new Promise((resolve) => {
        const modal = document.createElement('div');
        modal.className = 'modal show';
        modal.innerHTML = `
            <div class="modal-content" style="max-width: 500px;">
                <div class="modal-header">
                    <h2>Move to Humidor</h2>
                    <button class="close-btn" onclick="this.closest('.modal').remove(); event.stopPropagation();">&times;</button>
                </div>
                <div class="modal-body" style="padding: 0 2rem 2rem 2rem;">
                    <div class="form-group" style="margin-bottom: 1.5rem;">
                        <label for="humidorSelect" style="display: block; margin-bottom: 0.5rem; color: var(--text-primary); font-weight: 500;">Choose a humidor</label>
                        <select id="humidorSelect" class="form-control" style="width: 100%; padding: 0.75rem; border-radius: 0.5rem; border: 1px solid var(--border-color); background: var(--background-secondary); color: var(--text-primary);">
                            ${humidors.map(h => `<option value="${h.id}">${h.name}</option>`).join('')}
                        </select>
                    </div>
                    <div class="form-group" style="margin-bottom: 1.5rem;">
                        <label for="moveQuantity" style="display: block; margin-bottom: 0.5rem; color: var(--text-primary); font-weight: 500;">How many did you buy?</label>
                        <input type="number" id="moveQuantity" class="form-control" min="1" value="1" required style="width: 100%; padding: 0.75rem; border-radius: 0.5rem; border: 1px solid var(--border-color); background: var(--background-secondary); color: var(--text-primary);">
                    </div>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" onclick="this.closest('.modal').remove();">Cancel</button>
                    <button class="btn btn-primary" id="confirmMoveBtn">Move to Humidor</button>
                </div>
            </div>
        `;
        
        document.body.appendChild(modal);
        
        document.getElementById('confirmMoveBtn').addEventListener('click', () => {
            const selectedId = document.getElementById('humidorSelect').value;
            const quantity = parseInt(document.getElementById('moveQuantity').value) || 1;
            modal.remove();
            resolve({ humidorId: selectedId, quantity });
        });
        
        modal.querySelector('.close-btn').addEventListener('click', () => {
            resolve(null);
        });
        
        modal.querySelector('.btn-secondary').addEventListener('click', () => {
            resolve(null);
        });
    });
}

async function deleteWishListCigar(cigarId, event) {
    event?.stopPropagation();
    
    if (!confirm('Are you sure you want to remove this cigar from your wish list?')) {
        return;
    }
    
    try {
        // Remove from wish list (not deleting the cigar itself)
        await makeAuthenticatedRequest(`/api/v1/wish_list/${cigarId}`, {
            method: 'DELETE'
        });
        
        showToast('Cigar removed from wish list');
        await loadWishList();
    } catch (error) {
        console.error('Error removing cigar from wish list:', error);
        showToast('Failed to remove cigar from wish list', 'error');
    }
}

// ============================================
// USER MANAGEMENT FUNCTIONS
// ============================================

let users = [];
let usersCurrentPage = 1;
let usersTotalPages = 1;
let editingUserId = null;
let deleteUserId = null;
let passwordResetUserId = null;

// Check if current user is admin
function isCurrentUserAdmin() {
    const user = JSON.parse(localStorage.getItem('humidor_user') || '{}');
    return user.is_admin === true;
}

// Load users with pagination
async function loadUsers(page = 1, search = '') {
    console.log('[loadUsers] Starting, page:', page, 'search:', search);
    
    if (!isCurrentUserAdmin()) {
        console.log('[loadUsers] Not admin, skipping user load');
        return;
    }
    
    console.log('[loadUsers] User is admin, loading users...');

    try {
        let url = `/api/v1/admin/users?page=${page}&per_page=10`;
        if (search) {
            url += `&search=${encodeURIComponent(search)}`;
        }
        
        console.log('[loadUsers] Fetching from:', url);

        const response = await makeAuthenticatedRequest(url);
        
        console.log('[loadUsers] Response:', response);
        
        if (response && response.ok) {
            const data = await response.json();
            console.log('[loadUsers] Data received:', data);
            users = data.users || [];
            usersCurrentPage = page;
            usersTotalPages = Math.ceil((data.total || users.length) / 10);
            console.log('[loadUsers] Rendering', users.length, 'users');
            renderUsersTable();
            renderUsersPagination();
        } else {
            console.error('[loadUsers] Failed response:', response);
            showToast('Failed to load users', 'error');
        }
    } catch (error) {
        console.error('[loadUsers] Error loading users:', error);
        showToast('Error loading users', 'error');
    }
}

// Render users table
function renderUsersTable() {
    const tbody = document.getElementById('usersTableBody');
    
    if (!tbody) return; // Not on settings page
    
    if (users.length === 0) {
        tbody.innerHTML = `
            <tr>
                <td colspan="7" class="empty-users-state">
                    <i class="mdi mdi-account-off"></i>
                    <p>No users found</p>
                    <p style="font-size: 0.875rem;">Create a new user to get started.</p>
                </td>
            </tr>
        `;
        return;
    }
    
    const currentUser = JSON.parse(localStorage.getItem('humidor_user') || '{}');
    
    tbody.innerHTML = users.map(user => {
        const isCurrentUser = user.id === currentUser.id;
        const createdDate = new Date(user.created_at).toLocaleDateString();
        
        return `
            <tr>
                <td>
                    <div class="user-info">
                        <strong>${escapeHtml(user.username)}</strong>
                        ${isCurrentUser ? '<small style="color: var(--accent-gold);">(You)</small>' : ''}
                    </div>
                </td>
                <td>
                    <span class="user-email">${escapeHtml(user.email)}</span>
                </td>
                <td>${escapeHtml(user.full_name)}</td>
                <td>
                    ${user.is_admin ? 
                        '<span class="user-badge admin-badge"><i class="mdi mdi-shield-crown"></i> Admin</span>' : 
                        '<span class="user-badge user-badge-regular"><i class="mdi mdi-account"></i> User</span>'}
                </td>
                <td>
                    ${user.is_active ? 
                        '<span class="status-badge status-active"><i class="mdi mdi-check-circle"></i> Active</span>' : 
                        '<span class="status-badge status-inactive"><i class="mdi mdi-cancel"></i> Inactive</span>'}
                </td>
                <td>
                    <span class="user-created-date">${createdDate}</span>
                </td>
                <td>
                    <div class="user-actions">
                        <button class="btn btn-sm btn-edit" onclick="editUser('${user.id}')" title="Edit User">
                            <i class="mdi mdi-pencil"></i> Edit
                        </button>
                        <button class="btn btn-sm btn-reset-password" onclick="showPasswordResetDialog('${user.id}')" title="Reset Password">
                            <i class="mdi mdi-lock-reset"></i> Password
                        </button>
                        <button class="btn btn-sm btn-toggle" onclick="toggleUserActive('${user.id}')" title="${user.is_active ? 'Deactivate' : 'Activate'}">
                            <i class="mdi mdi-${user.is_active ? 'account-off' : 'account-check'}"></i> 
                            ${user.is_active ? 'Disable' : 'Enable'}
                        </button>
                        ${!isCurrentUser ? `
                            <button class="btn btn-sm btn-danger" onclick="showDeleteUserDialog('${user.id}')" title="Delete User">
                                <i class="mdi mdi-delete"></i> Delete
                            </button>
                        ` : ''}
                    </div>
                </td>
            </tr>
        `;
    }).join('');
}

// Render pagination
function renderUsersPagination() {
    const container = document.getElementById('usersPagination');
    if (!container || usersTotalPages <= 1) {
        if (container) container.innerHTML = '';
        return;
    }
    
    let html = '<div class="pagination">';
    
    // Previous button
    if (usersCurrentPage > 1) {
        html += `<button class="btn btn-secondary btn-sm" onclick="loadUsers(${usersCurrentPage - 1})">
            <i class="mdi mdi-chevron-left"></i> Previous
        </button>`;
    }
    
    // Page numbers
    html += `<span class="pagination-info">Page ${usersCurrentPage} of ${usersTotalPages}</span>`;
    
    // Next button
    if (usersCurrentPage < usersTotalPages) {
        html += `<button class="btn btn-secondary btn-sm" onclick="loadUsers(${usersCurrentPage + 1})">
            Next <i class="mdi mdi-chevron-right"></i>
        </button>`;
    }
    
    html += '</div>';
    container.innerHTML = html;
}

// Show create user modal
function showCreateUserModal() {
    console.log('[showCreateUserModal] Called');
    editingUserId = null;
    document.getElementById('userModalTitle').textContent = 'Create New User';
    document.getElementById('saveUserBtnText').textContent = 'Create User';
    document.getElementById('userForm').reset();
    document.getElementById('passwordGroup').style.display = 'block';
    document.getElementById('userPassword').required = true;
    document.getElementById('userIsActive').checked = true;
    document.getElementById('userModal').classList.add('active');
    console.log('[showCreateUserModal] Modal should be visible');
}

// Edit user
async function editUser(userId) {
    console.log('[editUser] Called with userId:', userId);
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users/${userId}`);
        
        if (response && response.ok) {
            const user = await response.json();
            editingUserId = userId;
            
            document.getElementById('userModalTitle').textContent = 'Edit User';
            document.getElementById('saveUserBtnText').textContent = 'Save Changes';
            document.getElementById('userUsername').value = user.username;
            document.getElementById('userEmail').value = user.email;
            document.getElementById('userFullName').value = user.full_name;
            document.getElementById('userIsAdmin').checked = user.is_admin;
            document.getElementById('userIsActive').checked = user.is_active;
            
            // Hide password field when editing
            document.getElementById('passwordGroup').style.display = 'none';
            document.getElementById('userPassword').required = false;
            document.getElementById('userPassword').value = '';
            
            document.getElementById('userModal').classList.add('active');
        } else {
            showToast('Failed to load user details', 'error');
        }
    } catch (error) {
        console.error('Error loading user:', error);
        showToast('Error loading user', 'error');
    }
}

// Handle user form submission
async function handleUserFormSubmit(e) {
    e.preventDefault();
    
    const formData = {
        username: document.getElementById('userUsername').value.trim(),
        email: document.getElementById('userEmail').value.trim(),
        full_name: document.getElementById('userFullName').value.trim(),
        is_admin: document.getElementById('userIsAdmin').checked,
        is_active: document.getElementById('userIsActive').checked
    };
    
    // Add password if creating or if provided when editing
    const password = document.getElementById('userPassword').value;
    if (!editingUserId || password) {
        if (password.length < 8) {
            showToast('Password must be at least 8 characters', 'error');
            return;
        }
        formData.password = password;
    }
    
    try {
        let response;
        if (editingUserId) {
            // Update existing user
            response = await makeAuthenticatedRequest(`/api/v1/admin/users/${editingUserId}`, {
                method: 'PUT',
                body: JSON.stringify(formData)
            });
        } else {
            // Create new user
            response = await makeAuthenticatedRequest('/api/v1/admin/users', {
                method: 'POST',
                body: JSON.stringify(formData)
            });
        }
        
        if (response && response.ok) {
            showToast(editingUserId ? 'User updated successfully' : 'User created successfully', 'success');
            closeUserModal();
            loadUsers(usersCurrentPage);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to save user', 'error');
        }
    } catch (error) {
        console.error('Error saving user:', error);
        showToast('Error saving user', 'error');
    }
}

// Toggle user active status
async function toggleUserActive(userId) {
    const user = users.find(u => u.id === userId);
    if (!user) return;
    
    const newStatus = !user.is_active;
    const action = newStatus ? 'activate' : 'deactivate';
    
    if (!confirm(`Are you sure you want to ${action} this user?`)) {
        return;
    }
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users/${userId}/active`, {
            method: 'PATCH',
            body: JSON.stringify({ is_active: newStatus })
        });
        
        if (response && response.ok) {
            showToast(`User ${action}d successfully`, 'success');
            loadUsers(usersCurrentPage);
        } else {
            showToast(`Failed to ${action} user`, 'error');
        }
    } catch (error) {
        console.error(`Error ${action}ing user:`, error);
        showToast(`Error ${action}ing user`, 'error');
    }
}

// Show delete user confirmation dialog
function showDeleteUserDialog(userId) {
    const user = users.find(u => u.id === userId);
    if (!user) return;
    
    deleteUserId = userId;
    document.getElementById('deleteUsername').textContent = user.username;
    document.getElementById('deleteUserEmail').textContent = user.email;
    document.getElementById('deleteUserConfirmModal').classList.add('active');
}

// Confirm delete user
async function confirmDeleteUser() {
    if (!deleteUserId) return;
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users/${deleteUserId}`, {
            method: 'DELETE'
        });
        
        if (response && response.ok) {
            showToast('User deleted successfully', 'success');
            closeDeleteUserModal();
            loadUsers(usersCurrentPage);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to delete user', 'error');
        }
    } catch (error) {
        console.error('Error deleting user:', error);
        showToast('Error deleting user', 'error');
    }
}

// Show password reset dialog
function showPasswordResetDialog(userId) {
    console.log('[showPasswordResetDialog] Called with userId:', userId);
    const user = users.find(u => u.id === userId);
    console.log('[showPasswordResetDialog] Found user:', user);
    if (!user) return;
    
    passwordResetUserId = userId;
    document.getElementById('passwordResetUsername').textContent = user.username;
    document.getElementById('userPasswordForm').reset();
    document.getElementById('userPasswordModal').classList.add('active');
    console.log('[showPasswordResetDialog] Modal should be visible');
}

// Handle password reset form submission
async function handlePasswordResetSubmit(e) {
    e.preventDefault();
    
    const newPassword = document.getElementById('newUserPassword').value;
    const confirmPassword = document.getElementById('confirmUserPassword').value;
    
    if (newPassword !== confirmPassword) {
        showToast('Passwords do not match', 'error');
        return;
    }
    
    if (newPassword.length < 8) {
        showToast('Password must be at least 8 characters', 'error');
        return;
    }
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users/${passwordResetUserId}/password`, {
            method: 'PATCH',
            body: JSON.stringify({ new_password: newPassword })
        });
        
        if (response && response.ok) {
            showToast('Password reset successfully', 'success');
            closePasswordResetModal();
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to reset password', 'error');
        }
    } catch (error) {
        console.error('Error resetting password:', error);
        showToast('Error resetting password', 'error');
    }
}

// Close user modal
function closeUserModal() {
    document.getElementById('userModal').classList.remove('active');
    editingUserId = null;
}

// Close delete user modal
function closeDeleteUserModal() {
    document.getElementById('deleteUserConfirmModal').classList.remove('active');
    deleteUserId = null;
}

// Close password reset modal
function closePasswordResetModal() {
    document.getElementById('userPasswordModal').classList.remove('active');
    passwordResetUserId = null;
}

// Show transfer ownership modal
async function showTransferOwnershipModal() {
    const modal = document.getElementById('transferOwnershipModal');
    
    // Load users for dropdowns
    try {
        const response = await makeAuthenticatedRequest('/api/v1/admin/users?per_page=1000');
        
        if (response && response.ok) {
            const data = await response.json();
            const fromSelect = document.getElementById('fromUserId');
            const toSelect = document.getElementById('toUserId');
            
            // Clear existing options except the first
            fromSelect.innerHTML = '<option value="">Select source user</option>';
            toSelect.innerHTML = '<option value="">Select destination user</option>';
            
            // Add user options
            data.users.forEach(user => {
                const fromOption = document.createElement('option');
                fromOption.value = user.id;
                fromOption.textContent = `${user.username} (${user.full_name})`;
                fromSelect.appendChild(fromOption);
                
                const toOption = document.createElement('option');
                toOption.value = user.id;
                toOption.textContent = `${user.username} (${user.full_name})`;
                toSelect.appendChild(toOption);
            });
            
            // Set up event listener to load humidors when from user changes
            fromSelect.addEventListener('change', loadHumidorsForTransfer);
            
            modal.classList.add('active');
        } else {
            showToast('Failed to load users', 'error');
        }
    } catch (error) {
        console.error('Error loading users for transfer:', error);
        showToast('Error loading users', 'error');
    }
}

// Load humidors for selected user in transfer modal
async function loadHumidorsForTransfer() {
    const fromUserId = document.getElementById('fromUserId').value;
    const humidorSelect = document.getElementById('transferHumidorId');
    
    // Reset humidor dropdown
    humidorSelect.innerHTML = '<option value="">All humidors from selected user</option>';
    
    if (!fromUserId) {
        humidorSelect.disabled = true;
        return;
    }
    
    try {
        // Fetch humidors for the selected user using admin endpoint
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users/${fromUserId}/humidors`);
        
        if (response && response.ok) {
            const humidors = await response.json();
            
            if (humidors.length === 0) {
                const option = document.createElement('option');
                option.value = '';
                option.textContent = 'User has no humidors';
                option.disabled = true;
                humidorSelect.appendChild(option);
                humidorSelect.disabled = true;
            } else {
                humidors.forEach(humidor => {
                    const option = document.createElement('option');
                    option.value = humidor.id;
                    const cigarCount = humidor.cigar_count || 0;
                    option.textContent = `${humidor.name} (${cigarCount} cigar${cigarCount !== 1 ? 's' : ''})`;
                    humidorSelect.appendChild(option);
                });
                humidorSelect.disabled = false;
            }
        }
    } catch (error) {
        console.error('Error loading humidors:', error);
        humidorSelect.disabled = true;
    }
}

// Handle transfer ownership form submission
async function handleTransferOwnershipSubmit(e) {
    e.preventDefault();
    console.log('[OWNERSHIP TRANSFER] Form submitted');
    
    const fromUserId = document.getElementById('fromUserId').value;
    const toUserId = document.getElementById('toUserId').value;
    const humidorId = document.getElementById('transferHumidorId').value;
    
    console.log('[OWNERSHIP TRANSFER] From:', fromUserId, 'To:', toUserId, 'Humidor:', humidorId);
    
    if (!fromUserId || !toUserId) {
        showToast('Please select both source and destination users', 'error');
        return;
    }
    
    if (fromUserId === toUserId) {
        showToast('Source and destination users must be different', 'error');
        return;
    }
    
    const confirmBtn = document.getElementById('confirmTransferOwnershipBtn');
    if (!confirmBtn) {
        console.error('[OWNERSHIP TRANSFER] Confirm button not found!');
        return;
    }
    
    const originalText = confirmBtn.innerHTML;
    confirmBtn.disabled = true;
    confirmBtn.innerHTML = '<span class="mdi mdi-loading mdi-spin"></span> Transferring...';
    
    try {
        const requestBody = {
            from_user_id: fromUserId,
            to_user_id: toUserId
        };
        
        // Only include humidor_id if a specific humidor is selected
        if (humidorId) {
            requestBody.humidor_id = humidorId;
        }
        
        console.log('[OWNERSHIP TRANSFER] Sending request:', requestBody);
        
        const response = await makeAuthenticatedRequest('/api/v1/admin/transfer-ownership', {
            method: 'POST',
            body: JSON.stringify(requestBody)
        });
        
        if (response && response.ok) {
            const result = await response.json();
            const transferType = humidorId ? 'humidor' : `${result.humidors_transferred} humidor(s)`;
            showToast(
                `Successfully transferred ${transferType} with ${result.cigars_transferred} cigar(s)`,
                'success'
            );
            document.getElementById('transferOwnershipModal').classList.remove('active');
            document.getElementById('transferOwnershipForm').reset();
            // Reset humidor dropdown
            document.getElementById('transferHumidorId').innerHTML = '<option value="">All humidors from selected user</option>';
            document.getElementById('transferHumidorId').disabled = true;
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to transfer ownership', 'error');
        }
    } catch (error) {
        console.error('Error transferring ownership:', error);
        showToast('Error transferring ownership', 'error');
    } finally {
        confirmBtn.disabled = false;
        confirmBtn.innerHTML = originalText;
    }
}

// User search with debounce
let userSearchTimeout;
function handleUserSearch(searchTerm) {
    clearTimeout(userSearchTimeout);
    userSearchTimeout = setTimeout(() => {
        loadUsers(1, searchTerm);
    }, 300);
}

// ============================================
// BACKUP & RESTORE FUNCTIONS
// ============================================

let backups = [];
let restoreTarget = null;
let deleteTarget = null;

async function loadBackups() {
    try {
        const response = await makeAuthenticatedRequest('/api/v1/backups');
        
        if (response && response.ok) {
            const data = await response.json();
            backups = data.backups || [];
            renderBackupsTable();
        } else {
            console.error('Failed to load backups');
            renderBackupsTable();
        }
    } catch (error) {
        console.error('Error loading backups:', error);
        renderBackupsTable();
    }
}

function renderBackupsTable() {
    const tbody = document.getElementById('backupsTableBody');
    
    if (!tbody) return; // Not on settings page
    
    if (backups.length === 0) {
        tbody.innerHTML = `
            <tr>
                <td colspan="4" class="empty-row">No backups found. Create your first backup above.</td>
            </tr>
        `;
        return;
    }
    
    tbody.innerHTML = backups.map(backup => `
        <tr>
            <td><span class="backup-name">${backup.name}</span></td>
            <td>${formatBackupDate(backup.date)}</td>
            <td>${backup.size}</td>
            <td class="actions-cell">
                <button class="btn-icon" onclick="downloadBackup('${backup.name}')" title="Download">
                    <span class="mdi mdi-download"></span>
                </button>
                <button class="btn-icon btn-restore" onclick="showRestoreDialog('${backup.name}')" title="Restore">
                    <span class="mdi mdi-database-import"></span>
                </button>
                <button class="btn-icon btn-delete" onclick="showDeleteDialog('${backup.name}')" title="Delete">
                    <span class="mdi mdi-delete"></span>
                </button>
            </td>
        </tr>
    `).join('');
}

function formatBackupDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

async function createBackup() {
    const btn = document.getElementById('createBackupBtn');
    if (!btn) return;
    
    btn.disabled = true;
    btn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Creating...';
    
    try {
        const response = await makeAuthenticatedRequest('/api/v1/backups', {
            method: 'POST'
        });
        
        if (response && response.ok) {
            showToast('Backup created successfully', 'success');
            await loadBackups();
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to create backup', 'error');
        }
    } catch (error) {
        console.error('Error creating backup:', error);
        showToast('Error creating backup', 'error');
    } finally {
        btn.disabled = false;
        btn.innerHTML = '<span class="mdi mdi-plus-circle"></span> Create Backup';
    }
}

async function uploadBackupFile(file) {
    if (!file.name.endsWith('.zip')) {
        showToast('Only .zip files are allowed', 'error');
        return;
    }
    
    const formData = new FormData();
    formData.append('file', file);
    
    try {
        const token = localStorage.getItem('humidor_token');
        const response = await fetch('/api/v1/backups/upload', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData
        });
        
        if (response.ok) {
            showToast('Backup uploaded successfully', 'success');
            await loadBackups();
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to upload backup', 'error');
        }
    } catch (error) {
        console.error('Error uploading backup:', error);
        showToast('Error uploading backup', 'error');
    }
}

async function downloadBackup(filename) {
    const token = localStorage.getItem('humidor_token');
    const url = `/api/v1/backups/${filename}/download`;
    
    try {
        const response = await fetch(url, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            throw new Error('Download failed');
        }

        // Get the blob from the response
        const blob = await response.blob();
        
        // Create a temporary link and trigger download
        const link = document.createElement('a');
        link.href = window.URL.createObjectURL(blob);
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        
        // Clean up the blob URL
        window.URL.revokeObjectURL(link.href);
        
        showToast('Backup downloaded successfully', 'success');
    } catch (error) {
        console.error('Error downloading backup:', error);
        showToast('Failed to download backup', 'error');
    }
}

function showRestoreDialog(filename) {
    restoreTarget = filename;
    const modal = document.getElementById('restoreConfirmModal');
    if (!modal) return;
    
    document.getElementById('restoreFilename').textContent = filename;
    document.getElementById('restoreConfirmCheckbox').checked = false;
    document.getElementById('confirmRestoreBtn').disabled = true;
    modal.classList.add('show');
}

function closeRestoreDialog() {
    const modal = document.getElementById('restoreConfirmModal');
    if (modal) {
        modal.classList.remove('show');
    }
    restoreTarget = null;
}

async function confirmRestore() {
    if (!restoreTarget) return;
    
    const btn = document.getElementById('confirmRestoreBtn');
    if (!btn) return;
    
    btn.disabled = true;
    btn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Restoring...';
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/backups/${restoreTarget}/restore`, {
            method: 'POST'
        });
        
        if (response && response.ok) {
            showToast('Backup restored successfully. Reloading...', 'success');
            closeRestoreDialog();
            
            // Reload the page after a short delay
            setTimeout(() => {
                window.location.reload();
            }, 1500);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to restore backup', 'error');
            btn.disabled = false;
            btn.innerHTML = '<span class="mdi mdi-database-import"></span> Restore Backup';
        }
    } catch (error) {
        console.error('Error restoring backup:', error);
        showToast('Error restoring backup', 'error');
        btn.disabled = false;
        btn.innerHTML = '<span class="mdi mdi-database-import"></span> Restore Backup';
    }
}

function showDeleteDialog(filename) {
    deleteTarget = filename;
    const modal = document.getElementById('deleteConfirmModal');
    if (!modal) return;
    
    document.getElementById('deleteFilename').textContent = filename;
    modal.classList.add('show');
}

function closeDeleteDialog() {
    const modal = document.getElementById('deleteConfirmModal');
    if (modal) {
        modal.classList.remove('show');
    }
    deleteTarget = null;
}

async function confirmDelete() {
    if (!deleteTarget) return;
    
    const btn = document.getElementById('confirmDeleteBtn');
    if (!btn) return;
    
    btn.disabled = true;
    btn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Deleting...';
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/backups/${deleteTarget}`, {
            method: 'DELETE'
        });
        
        if (response && response.ok) {
            showToast('Backup deleted successfully', 'success');
            closeDeleteDialog();
            await loadBackups();
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to delete backup', 'error');
        }
    } catch (error) {
        console.error('Error deleting backup:', error);
        showToast('Error deleting backup', 'error');
    } finally {
        btn.disabled = false;
        btn.innerHTML = '<span class="mdi mdi-delete"></span> Delete Backup';
    }
}

// Initialize user management event listeners
function initializeUserManagementHandlers() {
    // Create user button
    const createUserBtn = document.getElementById('createUserBtn');
    if (createUserBtn) {
        createUserBtn.addEventListener('click', showCreateUserModal);
    }
    
    // Transfer ownership button
    const transferOwnershipBtn = document.getElementById('transferOwnershipBtn');
    if (transferOwnershipBtn) {
        transferOwnershipBtn.addEventListener('click', showTransferOwnershipModal);
    }
    
    // User search input
    const userSearchInput = document.getElementById('userSearchInput');
    if (userSearchInput) {
        userSearchInput.addEventListener('input', (e) => {
            handleUserSearch(e.target.value);
        });
    }
    
    // User form submission
    const userForm = document.getElementById('userForm');
    if (userForm) {
        userForm.addEventListener('submit', handleUserFormSubmit);
    }
    
    // User modal close handlers
    const closeUserModal = document.getElementById('closeUserModal');
    const cancelUserBtn = document.getElementById('cancelUserBtn');
    
    if (closeUserModal) closeUserModal.addEventListener('click', () => {
        document.getElementById('userModal').classList.remove('active');
        editingUserId = null;
    });
    
    if (cancelUserBtn) cancelUserBtn.addEventListener('click', () => {
        document.getElementById('userModal').classList.remove('active');
        editingUserId = null;
    });
    
    // Delete user dialog handlers
    const closeDeleteUserModal = document.getElementById('closeDeleteUserModal');
    const cancelDeleteUserBtn = document.getElementById('cancelDeleteUserBtn');
    const confirmDeleteUserBtn = document.getElementById('confirmDeleteUserBtn');
    
    if (closeDeleteUserModal) closeDeleteUserModal.addEventListener('click', () => {
        document.getElementById('deleteUserConfirmModal').classList.remove('active');
        deleteUserId = null;
    });
    
    if (cancelDeleteUserBtn) cancelDeleteUserBtn.addEventListener('click', () => {
        document.getElementById('deleteUserConfirmModal').classList.remove('active');
        deleteUserId = null;
    });
    
    if (confirmDeleteUserBtn) confirmDeleteUserBtn.addEventListener('click', confirmDeleteUser);
    
    // Password reset dialog handlers
    const closePasswordModal = document.getElementById('closeUserPasswordModal');
    const cancelPasswordBtn = document.getElementById('cancelPasswordBtn');
    const userPasswordForm = document.getElementById('userPasswordForm');
    
    if (closePasswordModal) closePasswordModal.addEventListener('click', () => {
        document.getElementById('userPasswordModal').classList.remove('active');
        passwordResetUserId = null;
    });
    
    if (cancelPasswordBtn) cancelPasswordBtn.addEventListener('click', () => {
        document.getElementById('userPasswordModal').classList.remove('active');
        passwordResetUserId = null;
    });
    
    if (userPasswordForm) userPasswordForm.addEventListener('submit', handlePasswordResetSubmit);
    
    // Transfer ownership dialog handlers
    const closeTransferOwnershipModal = document.getElementById('closeTransferModal');
    const cancelTransferOwnershipBtn = document.getElementById('cancelTransferOwnershipBtn');
    const transferOwnershipForm = document.getElementById('transferOwnershipForm');
    
    if (closeTransferOwnershipModal) closeTransferOwnershipModal.addEventListener('click', () => {
        document.getElementById('transferOwnershipModal').classList.remove('active');
    });
    
    if (cancelTransferOwnershipBtn) cancelTransferOwnershipBtn.addEventListener('click', () => {
        document.getElementById('transferOwnershipModal').classList.remove('active');
    });
    
    if (transferOwnershipForm) transferOwnershipForm.addEventListener('submit', handleTransferOwnershipSubmit);
}

// Initialize backup event listeners
function initializeBackupHandlers() {
    // Create backup button
    const createBtn = document.getElementById('createBackupBtn');
    if (createBtn) {
        createBtn.addEventListener('click', createBackup);
    }
    
    // Upload backup input
    const uploadInput = document.getElementById('uploadBackupInput');
    if (uploadInput) {
        uploadInput.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                uploadBackupFile(e.target.files[0]);
                e.target.value = ''; // Reset input
            }
        });
    }
    
    // Restore dialog handlers
    const closeRestoreBtn = document.getElementById('closeRestoreModal');
    const cancelRestoreBtn = document.getElementById('cancelRestoreBtn');
    const confirmRestoreBtn = document.getElementById('confirmRestoreBtn');
    const restoreCheckbox = document.getElementById('restoreConfirmCheckbox');
    
    if (closeRestoreBtn) closeRestoreBtn.addEventListener('click', closeRestoreDialog);
    if (cancelRestoreBtn) cancelRestoreBtn.addEventListener('click', closeRestoreDialog);
    if (confirmRestoreBtn) confirmRestoreBtn.addEventListener('click', confirmRestore);
    if (restoreCheckbox) {
        restoreCheckbox.addEventListener('change', (e) => {
            if (confirmRestoreBtn) {
                confirmRestoreBtn.disabled = !e.target.checked;
            }
        });
    }
    
    // Delete dialog handlers
    const closeDeleteBtn = document.getElementById('closeDeleteModal');
    const cancelDeleteBtn = document.getElementById('cancelDeleteBtn');
    const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
    
    if (closeDeleteBtn) closeDeleteBtn.addEventListener('click', closeDeleteDialog);
    if (cancelDeleteBtn) cancelDeleteBtn.addEventListener('click', closeDeleteDialog);
    if (confirmDeleteBtn) confirmDeleteBtn.addEventListener('click', confirmDelete);
}

// ============================================
// SHARE HUMIDOR FUNCTIONALITY
// ============================================

let currentShareHumidorId = null;
let availableUsers = [];

async function loadAvailableUsers() {
    try {
        // Load all users from admin endpoint
        const response = await makeAuthenticatedRequest(`/api/v1/admin/users?page=1&per_page=1000`);
        
        if (!response.ok) {
            throw new Error('Failed to load users');
        }
        
        const data = await response.json();
        availableUsers = data.users || [];
        return availableUsers;
    } catch (error) {
        console.error('Error loading users:', error);
        return [];
    }
}

function populateUserDropdown(humidorShares = []) {
    const userSelect = document.getElementById('shareUserSelect');
    const currentUserId = localStorage.getItem('userId'); // We'll need to get this from auth
    
    // Clear existing options except the first one
    userSelect.innerHTML = '<option value="">-- Select a user to share with --</option>';
    
    // Get list of already shared user IDs
    const sharedUserIds = humidorShares.map(share => share.shared_with_user.id);
    
    // Filter out current user and already shared users
    const filteredUsers = availableUsers.filter(user => 
        user.id !== currentUserId && 
        !sharedUserIds.includes(user.id)
    );
    
    // Add users to dropdown
    filteredUsers.forEach(user => {
        const option = document.createElement('option');
        option.value = user.id;
        option.textContent = `${user.username}${user.full_name ? ' (' + user.full_name + ')' : ''} - ${user.email}`;
        userSelect.appendChild(option);
    });
}

async function openShareHumidorModal(humidorId, humidorName) {
    currentShareHumidorId = humidorId;
    const modal = document.getElementById('shareHumidorModal');
    const nameInput = document.getElementById('shareHumidorName');
    const userSelect = document.getElementById('shareUserSelect');
    
    nameInput.value = humidorName;
    userSelect.value = '';
    
    // Load available users if not already loaded
    if (availableUsers.length === 0) {
        await loadAvailableUsers();
    }
    
    // Load current shares and populate dropdown
    await loadCurrentShares(humidorId);
    
    // Load public share status
    await loadPublicShare(humidorId);
    
    modal.style.display = 'flex';
}

function closeShareHumidorModal() {
    const modal = document.getElementById('shareHumidorModal');
    modal.style.display = 'none';
    currentShareHumidorId = null;
}

async function loadCurrentShares(humidorId) {
    console.log('=== loadCurrentShares() called ===');
    console.log('humidorId:', humidorId);
    
    const url = `/api/v1/humidors/${humidorId}/shares`;
    console.log('Fetching from URL:', url);
    
    try {
        const response = await makeAuthenticatedRequest(url);
        
        console.log('Shares response status:', response.status);
        
        if (!response.ok) {
            throw new Error('Failed to load shares');
        }
        
        const data = await response.json();
        console.log('Shares data received:', data);
        console.log('Data type:', typeof data);
        console.log('Is array?:', Array.isArray(data));
        console.log('Data keys:', Object.keys(data));
        console.log('data.shares:', data.shares);
        console.log('Number of shares:', data.shares ? data.shares.length : 0);
        
        const sharesList = document.getElementById('currentSharesList');
        console.log('sharesList element:', sharesList);
        
        if (data.shares && data.shares.length > 0) {
            console.log('Rendering', data.shares.length, 'shares');
            sharesList.innerHTML = data.shares.map(share => `
                <div class="share-item">
                    <div class="share-user-info">
                        <div class="username">${escapeHtml(share.shared_with_user.username)}</div>
                        <div class="email">${escapeHtml(share.shared_with_user.email)}</div>
                        ${share.shared_with_user.full_name ? `<div class="email">${escapeHtml(share.shared_with_user.full_name)}</div>` : ''}
                        <span class="permission-badge">${share.permission_level.toUpperCase()}</span>
                    </div>
                    <div class="share-actions">
                        <select onchange="updateSharePermission('${humidorId}', '${share.shared_with_user.id}', this.value)">
                            <option value="view" ${share.permission_level === 'view' ? 'selected' : ''}>View</option>
                            <option value="edit" ${share.permission_level === 'edit' ? 'selected' : ''}>Edit</option>
                            <option value="full" ${share.permission_level === 'full' ? 'selected' : ''}>Full</option>
                        </select>
                        <button onclick="revokeShare('${humidorId}', '${share.shared_with_user.id}')">
                            <i class="mdi mdi-delete"></i> Remove
                        </button>
                    </div>
                </div>
            `).join('');
            
            // Repopulate dropdown excluding newly loaded shares
            populateUserDropdown(data.shares);
        } else {
            console.log('No shares found, showing empty message');
            sharesList.innerHTML = '<p class="text-muted">This humidor is not currently shared with anyone.</p>';
            
            // Populate dropdown with all available users
            populateUserDropdown([]);
        }
        
        console.log('=== loadCurrentShares() complete ===');
    } catch (error) {
        console.error('Error loading shares:', error);
        showToast('Failed to load shares. Please try again.', 'error');
    }
}

async function shareHumidor() {
    console.log('=== shareHumidor() called ===');
    console.log('currentShareHumidorId:', currentShareHumidorId);
    
    const userSelect = document.getElementById('shareUserSelect');
    const selectedUserId = userSelect.value;
    
    console.log('selectedUserId:', selectedUserId);
    
    if (!selectedUserId) {
        showToast('Please select a user to share with', 'error');
        return;
    }
    
    const permissionLevel = document.getElementById('sharePermissionLevel').value;
    console.log('permissionLevel:', permissionLevel);
    
    try {
        const url = `/api/v1/humidors/${currentShareHumidorId}/share`;
        const body = {
            user_id: selectedUserId,
            permission_level: permissionLevel
        };
        
        console.log('Making POST request to:', url);
        console.log('Request body:', body);
        
        const response = await makeAuthenticatedRequest(url, {
            method: 'POST',
            body: JSON.stringify(body)
        });
        
        console.log('Response status:', response.status);
        console.log('Response ok:', response.ok);
        
        if (!response.ok) {
            const error = await response.json();
            console.error('Share failed with error:', error);
            throw new Error(error.error || 'Failed to share humidor');
        }
        
        const result = await response.json();
        console.log('Share successful, result:', result);
        
        showToast('Humidor shared successfully!', 'success');
        
        // Reset selection
        userSelect.value = '';
        document.getElementById('sharePermissionLevel').value = 'edit';
        
        // Reload shares list (which will also update the dropdown)
        console.log('Reloading shares list...');
        await loadCurrentShares(currentShareHumidorId);
        console.log('Shares list reloaded');
    } catch (error) {
        console.error('Error sharing humidor:', error);
        showToast(`Failed to share humidor: ${error.message}`, 'error');
    }
}

async function updateSharePermission(humidorId, userId, newPermission) {
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/humidors/${humidorId}/share/${userId}`, {
            method: 'PATCH',
            body: JSON.stringify({
                permission_level: newPermission
            })
        });
        
        if (!response.ok) {
            throw new Error('Failed to update permission');
        }
        
        showToast('Permission updated successfully!', 'success');
        await loadCurrentShares(humidorId);
    } catch (error) {
        console.error('Error updating permission:', error);
        showToast('Failed to update permission. Please try again.', 'error');
    }
}

async function revokeShare(humidorId, userId) {
    if (!confirm('Are you sure you want to revoke access for this user?')) {
        return;
    }
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/humidors/${humidorId}/share/${userId}`, {
            method: 'DELETE'
        });
        
        if (!response.ok) {
            throw new Error('Failed to revoke share');
        }
        
        showToast('Access revoked successfully!', 'success');
        await loadCurrentShares(humidorId);
    } catch (error) {
        console.error('Error revoking share:', error);
        showToast('Failed to revoke access. Please try again.', 'error');
    }
}

// ========================================
// PUBLIC SHARE FUNCTIONS
// ========================================

async function loadPublicShare(humidorId) {
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-shares`,
            { method: 'GET' }
        );
        
        if (response.ok) {
            const shares = await response.json();
            if (shares && shares.length > 0) {
                showActivePublicShares(shares);
            } else {
                showNoPublicShare();
            }
        } else {
            console.error('Failed to load public shares');
            showNoPublicShare();
        }
    } catch (error) {
        console.error('Error loading public shares:', error);
        showNoPublicShare();
    }
}

function showNoPublicShare() {
    document.getElementById('noPublicShare').style.display = 'block';
    document.getElementById('activePublicShare').style.display = 'none';
    
    // Set default expiry to 30 days from now
    const defaultExpiry = new Date();
    defaultExpiry.setDate(defaultExpiry.getDate() + 30);
    const expiryInput = document.getElementById('publicShareExpiry');
    if (expiryInput) {
        expiryInput.value = defaultExpiry.toISOString().slice(0, 16);
    }
    
    // Clear label input
    const labelInput = document.getElementById('publicShareLabel');
    if (labelInput) {
        labelInput.value = '';
    }
    
    // Setup checkbox toggle
    setupExpiryCheckbox();
}

function showActivePublicShares(shares) {
    document.getElementById('noPublicShare').style.display = 'none';
    document.getElementById('activePublicShare').style.display = 'block';
    
    const container = document.getElementById('publicSharesList');
    if (!container) return;
    
    container.innerHTML = shares.map(share => {
        const shareUrl = `${window.location.protocol}//${window.location.host}/shared/humidors/${share.token_id}`;
        
        let expiryInfo = '';
        if (share.expires_at) {
            const expiryDate = new Date(share.expires_at);
            const now = new Date();
            const daysUntilExpiry = Math.ceil((expiryDate - now) / (1000 * 60 * 60 * 24));
            
            expiryInfo = `
                <div class="share-info-item">
                    <span class="mdi mdi-clock-outline"></span>
                    Expires: <strong>${expiryDate.toLocaleString()}</strong>
                    ${daysUntilExpiry <= 7 ? `<span class="warning-badge">${daysUntilExpiry} days left</span>` : ''}
                </div>
            `;
        } else {
            expiryInfo = `
                <div class="share-info-item">
                    <span class="mdi mdi-infinity"></span>
                    <strong>Never expires</strong>
                </div>
            `;
        }
        
        const includeInfo = `
            <div class="share-info-item">
                <span class="mdi mdi-heart${share.include_favorites ? '' : '-outline'}"></span>
                Favorites: ${share.include_favorites ? 'Yes' : 'No'}
            </div>
            <div class="share-info-item">
                <span class="mdi mdi-book${share.include_wish_list ? '' : '-outline'}"></span>
                Wish List: ${share.include_wish_list ? 'Yes' : 'No'}
            </div>
        `;
        
        return `
            <div class="public-share-item" data-token="${share.token_id}">
                ${share.label ? `<div class="share-label">${escapeHtml(share.label)}</div>` : ''}
                <div class="share-link-box">
                    <input type="text" value="${shareUrl}" readonly class="share-url-input" />
                    <button class="btn-icon" onclick="copyShareLink('${shareUrl}')" title="Copy Link">
                        <span class="mdi mdi-content-copy"></span>
                    </button>
                </div>
                <div class="share-info">
                    ${expiryInfo}
                    ${includeInfo}
                </div>
                <button class="btn btn-danger btn-sm" onclick="deletePublicShare('${share.token_id}')">
                    <span class="mdi mdi-delete"></span>
                    Delete
                </button>
            </div>
        `;
    }).join('');
}

window.copyShareLink = async function(url) {
    try {
        await navigator.clipboard.writeText(url);
        showToast('Link copied to clipboard!', 'success');
    } catch (error) {
        showToast('Failed to copy link', 'error');
    }
}

function setupExpiryCheckbox() {
    const checkbox = document.getElementById('neverExpiresCheckbox');
    const expiryGroup = document.getElementById('expiryDateGroup');
    
    if (!checkbox || !expiryGroup) return;
    
    // Remove existing listeners
    const newCheckbox = checkbox.cloneNode(true);
    checkbox.parentNode.replaceChild(newCheckbox, checkbox);
    
    newCheckbox.addEventListener('change', (e) => {
        if (e.target.checked) {
            expiryGroup.style.display = 'none';
        } else {
            expiryGroup.style.display = 'block';
        }
    });
    
    // Initialize state
    expiryGroup.style.display = newCheckbox.checked ? 'none' : 'block';
}

async function createPublicShare() {
    const humidorId = currentShareHumidorId;
    const neverExpires = document.getElementById('neverExpiresCheckbox').checked;
    const includeFavorites = document.getElementById('includeFavoritesCheckbox').checked;
    const includeWishList = document.getElementById('includeWishListCheckbox').checked;
    const expiryInput = document.getElementById('publicShareExpiry').value;
    const labelInput = document.getElementById('publicShareLabel');
    const label = labelInput ? labelInput.value.trim() : '';
    
    let requestBody = { 
        never_expires: neverExpires,
        include_favorites: includeFavorites,
        include_wish_list: includeWishList,
        label: label || null
    };
    
    if (!neverExpires && expiryInput) {
        requestBody.expires_at = new Date(expiryInput).toISOString();
    }
    
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-share`,
            {
                method: 'POST',
                body: JSON.stringify(requestBody)
            }
        );
        
        if (response.ok) {
            showToast('Public share link created!', 'success');
            await loadPublicShare(humidorId);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to create share link', 'error');
        }
    } catch (error) {
        console.error('Failed to create public share:', error);
        showToast('Failed to create share link', 'error');
    }
}

async function deletePublicShare(tokenId) {
    if (!confirm('Are you sure you want to delete this share link? This link will stop working immediately.')) {
        return;
    }
    
    const humidorId = currentShareHumidorId;
    
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-shares/${tokenId}`,
            { method: 'DELETE' }
        );
        
        if (response.ok) {
            showToast('Share link deleted', 'success');
            await loadPublicShare(humidorId);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to delete link', 'error');
        }
    } catch (error) {
        console.error('Failed to delete public share:', error);
        showToast('Failed to delete link', 'error');
    }
}

window.deletePublicShare = deletePublicShare;

async function revokeAllPublicShares() {
    if (!confirm('Are you sure you want to revoke ALL public share links for this humidor? All links will stop working immediately.')) {
        return;
    }
    
    const humidorId = currentShareHumidorId;
    
    try {
        const response = await makeAuthenticatedRequest(
            `/api/v1/humidors/${humidorId}/public-share`,
            { method: 'DELETE' }
        );
        
        if (response.ok) {
            showToast('All share links revoked', 'success');
            await loadPublicShare(humidorId);
        } else {
            const error = await response.json();
            showToast(error.message || 'Failed to revoke links', 'error');
        }
    } catch (error) {
        console.error('Failed to revoke public shares:', error);
        showToast('Failed to revoke links', 'error');
    }
}

// Setup share modal event listeners
function setupShareModalListeners() {
    const closeBtn = document.getElementById('shareHumidorClose');
    const addUserBtn = document.getElementById('shareAddUserBtn');
    const modal = document.getElementById('shareHumidorModal');
    
    // Public share create button
    const createPublicBtn = document.getElementById('createPublicShareBtn');
    
    if (closeBtn) {
        closeBtn.addEventListener('click', closeShareHumidorModal);
    }
    
    if (addUserBtn) {
        addUserBtn.addEventListener('click', shareHumidor);
    }
    
    if (createPublicBtn) {
        createPublicBtn.addEventListener('click', createPublicShare);
    }
    
    // Setup expiry checkbox behavior
    setupExpiryCheckbox();
    
    // Close modal when clicking outside
    if (modal) {
        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                closeShareHumidorModal();
            }
        });
    }
}

// Call setup when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', setupShareModalListeners);
} else {
    setupShareModalListeners();
}

// Global functions for modal opening (called from HTML)
function openBrandModal() { openOrganizerModal('brand'); }
function openSizeModal() { openOrganizerModal('size'); }
function openOriginModal() { openOrganizerModal('origin'); }
function openStrengthModal() { openOrganizerModal('strength'); }
function openRingGaugeModal() { openOrganizerModal('ringGauge'); }

// Public Share View Functions
async function initializePublicShareView() {
    console.log('[Public Share] Initializing public share view');
    
    // Initialize theme
    initializeTheme();
    setupThemeToggle();
    
    // Hide authenticated-only elements
    document.querySelectorAll('.auth-only, .user-info, .add-btn, .add-humidor-btn-primary, #addCigarBtnNav, #addHumidorBtnSidebar').forEach(el => {
        el.style.display = 'none';
    });
    
    // Show public share indicator
    const sidebar = document.querySelector('.sidebar-footer');
    if (sidebar) {
        sidebar.innerHTML = `
            <p class="text-muted small" style="text-align: center; padding: 1rem;">
                <span class="mdi mdi-eye-outline"></span> Public Read-Only View
            </p>
            <a href="/login.html" class="btn btn-secondary btn-sm" style="width: 100%;">
                <span class="mdi mdi-login"></span> Login to Your Account
            </a>
        `;
    }
    
    // Extract token from URL
    const pathParts = window.location.pathname.split('/');
    const token = pathParts[pathParts.length - 1];
    
    try {
        // Fetch public humidor data
        const response = await fetch(`/api/v1/shared/humidors/${token}`);
        
        if (!response.ok) {
            showPublicShareError();
            return;
        }
        
        const data = await response.json();
        console.log('[Public Share] Loaded data:', data);
        
        // Store data globally for card rendering
        window.publicShareData = data;
        window.cigars = data.cigars || [];
        window.favorites = data.favorites || [];
        window.wishListCigars = data.wish_list || [];
        
        // Hide welcome section, show main content
        const welcomeSection = document.getElementById('welcomeSection');
        const mainContentSection = document.getElementById('mainContentSection');
        if (welcomeSection) welcomeSection.style.display = 'none';
        if (mainContentSection) mainContentSection.style.display = 'block';
        
        // Render the humidor with public data
        renderPublicHumidor(data);
        
    } catch (error) {
        console.error('[Public Share] Error loading:', error);
        showPublicShareError();
    }
}

function renderPublicHumidor(data) {
    const container = document.getElementById('humidorsContainer');
    if (!container) return;
    
    // Build humidor HTML using same structure as authenticated view
    const humidorHTML = `
        <div class="humidor-section" data-humidor-id="${data.id}">
            <div class="humidor-header">
                <div class="humidor-info-card">
                    <div class="humidor-title-section">
                        <i class="mdi mdi-home-variant"></i>
                        <div>
                            <h2 class="humidor-name">${escapeHtml(data.name)}</h2>
                            ${data.description ? `<p class="humidor-location">${escapeHtml(data.description)}</p>` : ''}
                            <span class="permission-badge" style="background: #3498db; color: white; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; margin-left: 8px;">SHARED - VIEW ONLY</span>
                        </div>
                    </div>
                </div>
                <div class="humidor-stats">
                    <div class="stat-card">
                        <i class="mdi mdi-cigar stat-icon"></i>
                        <div>
                            <div class="stat-value">${data.cigars.reduce((sum, c) => sum + c.quantity, 0)}</div>
                            <div class="stat-label">Total Cigars</div>
                        </div>
                    </div>
                    <div class="stat-card">
                        <i class="mdi mdi-format-list-bulleted stat-icon"></i>
                        <div>
                            <div class="stat-value">${data.cigar_count}</div>
                            <div class="stat-label">Cigar Types</div>
                        </div>
                    </div>
                </div>
            </div>
            ${data.cigars.length > 0 ? `
                <div class="cigars-grid">
                    ${data.cigars.map(cigar => createPublicCigarCard(cigar)).join('')}
                </div>
            ` : `
                <div class="empty-state">
                    <i class="mdi mdi-cigar-off"></i>
                    <p>No cigars in this humidor</p>
                </div>
            `}
        </div>
    `;
    
    container.innerHTML = humidorHTML;
    
    // Update navigation if favorites/wishlist are included
    updatePublicNavigation(data);
}

function createPublicCigarCard(cigar) {
    const imageHtml = cigar.image_url 
        ? `<img src="${cigar.image_url}" alt="${escapeHtml(cigar.name)}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
           <img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="display: none; width: 100%; height: 100%; object-fit: contain; padding: 2rem;">` 
        : `<img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`;
    
    return `
        <div class="cigar-card" data-cigar-id="${cigar.id}" onclick="openPublicCigarDetails('${cigar.id}')" style="cursor: pointer;">
            <div class="cigar-card-image">
                ${imageHtml}
            </div>
            <div class="cigar-card-content">
                ${cigar.brand ? `<div class="cigar-card-brand">${escapeHtml(cigar.brand)}</div>` : ''}
                <h3 class="cigar-card-name">${escapeHtml(cigar.name)}</h3>
                <div class="cigar-card-quantity">
                    <span class="quantity-value">${cigar.quantity}</span>
                </div>
            </div>
        </div>
    `;
}

function openPublicCigarDetails(cigarId) {
    // Find cigar in all arrays
    let cigar = window.cigars.find(c => c.id === cigarId) || 
                window.favorites.find(c => c.id === cigarId) || 
                window.wishListCigars.find(c => c.id === cigarId);
    
    if (!cigar) return;
    
    const modal = document.getElementById('reportCardModal');
    if (!modal) return;
    
    // Populate modal
    document.getElementById('reportCardImage').src = cigar.image_url || '/static/cigar-placeholder.png';
    document.getElementById('reportCardBrand').textContent = cigar.brand || '-';
    document.getElementById('reportCardName').textContent = cigar.name;
    document.getElementById('reportCardOrigin').textContent = cigar.origin || '-';
    document.getElementById('reportCardStrength').textContent = cigar.strength || '-';
    document.getElementById('reportCardRingGauge').textContent = cigar.ring_gauge || '-';
    
    const lengthElem = document.getElementById('reportCardSize');
    if (lengthElem) lengthElem.textContent = cigar.length_inches ? `${cigar.length_inches}"` : '-';
    
    document.getElementById('reportCardQuantity').textContent = cigar.quantity || '-';
    document.getElementById('reportCardNotes').textContent = cigar.notes || 'No notes available';
    
    // Hide action buttons for public view
    const actionsContainer = document.querySelector('.report-card-actions');
    if (actionsContainer) actionsContainer.style.display = 'none';
    
    modal.classList.add('show');
}

function updatePublicNavigation(data) {
    // Hide organizers dropdown section
    const organizersToggle = document.getElementById('organizersToggle');
    if (organizersToggle) {
        const organizersSection = organizersToggle.closest('.nav-section');
        if (organizersSection) {
            organizersSection.style.display = 'none';
        }
    }
    
    // Hide admin section if exists
    document.querySelectorAll('.nav-section').forEach(section => {
        const title = section.querySelector('.nav-section-title');
        if (title && title.textContent.includes('Admin')) {
            section.style.display = 'none';
        }
    });
    
    // Hide settings button
    const settingsButton = document.querySelector('.nav-item[data-page="settings"]');
    if (settingsButton) {
        settingsButton.style.display = 'none';
    }
    
    // Update Collections section title
    const collectionsTitle = document.querySelector('.nav-section-title');
    if (collectionsTitle && collectionsTitle.textContent === 'Collections') {
        collectionsTitle.textContent = 'SHARED VIEW';
    }
    
    // Show/hide favorites nav item based on data
    const favoritesNav = document.querySelector('.nav-item[data-page="favorites"]');
    if (favoritesNav) {
        if (data.favorites && data.favorites.length > 0) {
            favoritesNav.style.display = 'flex';
        } else {
            favoritesNav.style.display = 'none';
        }
    }
    
    // Show/hide wishlist nav item based on data
    const wishlistNav = document.querySelector('.nav-item[data-page="wishlist"]');
    if (wishlistNav) {
        if (data.wish_list && data.wish_list.length > 0) {
            wishlistNav.style.display = 'flex';
        } else {
            wishlistNav.style.display = 'none';
        }
    }
    
    // Set up nav click handlers for public view
    setupPublicNavigation(data);
}

function setupPublicNavigation(data) {
    // Handle humidor nav click - show main humidor view
    const humidorNav = document.querySelector('.nav-item[data-page="humidors"]');
    if (humidorNav) {
        humidorNav.addEventListener('click', (e) => {
            e.preventDefault();
            showPublicHumidorView(data);
        });
    }
    
    // Handle favorites nav click
    const favoritesNav = document.querySelector('.nav-item[data-page="favorites"]');
    if (favoritesNav && data.favorites && data.favorites.length > 0) {
        favoritesNav.addEventListener('click', (e) => {
            e.preventDefault();
            showPublicFavoritesView(data);
        });
    }
    
    // Handle wishlist nav click
    const wishlistNav = document.querySelector('.nav-item[data-page="wishlist"]');
    if (wishlistNav && data.wish_list && data.wish_list.length > 0) {
        wishlistNav.addEventListener('click', (e) => {
            e.preventDefault();
            showPublicWishlistView(data);
        });
    }
    
    // Set up modal close button
    const closeButton = document.getElementById('closeReportCardModal');
    if (closeButton) {
        closeButton.addEventListener('click', () => {
            const modal = document.getElementById('reportCardModal');
            if (modal) modal.classList.remove('show');
        });
    }
    
    // Close modal on background click
    const modal = document.getElementById('reportCardModal');
    if (modal) {
        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                modal.classList.remove('show');
            }
        });
    }
}

function showPublicHumidorView(data) {
    // Update nav active state
    document.querySelectorAll('.nav-item').forEach(item => item.classList.remove('active'));
    const humidorNav = document.querySelector('.nav-item[data-page="humidors"]');
    if (humidorNav) humidorNav.classList.add('active');
    
    // Render humidor content
    renderPublicHumidor(data);
}

function showPublicFavoritesView(data) {
    // Update nav active state
    document.querySelectorAll('.nav-item').forEach(item => item.classList.remove('active'));
    const favoritesNav = document.querySelector('.nav-item[data-page="favorites"]');
    if (favoritesNav) favoritesNav.classList.add('active');
    
    // Render favorites
    const container = document.getElementById('humidorsContainer');
    if (!container) return;
    
    const favoritesHTML = `
        <div class="favorites-section">
            <div class="section-header">
                <h2><i class="mdi mdi-star"></i> Favorites</h2>
                <p style="color: #999; margin-top: 0.5rem;">Shared favorites from this humidor</p>
            </div>
            ${data.favorites.length > 0 ? `
                <div class="cigars-grid">
                    ${data.favorites.map(cigar => createPublicCigarCard(cigar)).join('')}
                </div>
            ` : `
                <div class="empty-state">
                    <i class="mdi mdi-star-off"></i>
                    <p>No favorites shared</p>
                </div>
            `}
        </div>
    `;
    
    container.innerHTML = favoritesHTML;
}

function showPublicWishlistView(data) {
    // Update nav active state
    document.querySelectorAll('.nav-item').forEach(item => item.classList.remove('active'));
    const wishlistNav = document.querySelector('.nav-item[data-page="wishlist"]');
    if (wishlistNav) wishlistNav.classList.add('active');
    
    // Render wishlist
    const container = document.getElementById('humidorsContainer');
    if (!container) return;
    
    const wishlistHTML = `
        <div class="wishlist-section">
            <div class="section-header">
                <h2><i class="mdi mdi-playlist-star"></i> Wish List</h2>
                <p style="color: #999; margin-top: 0.5rem;">Shared wish list from this humidor</p>
            </div>
            ${data.wish_list.length > 0 ? `
                <div class="cigars-grid">
                    ${data.wish_list.map(cigar => createPublicCigarCard(cigar)).join('')}
                </div>
            ` : `
                <div class="empty-state">
                    <i class="mdi mdi-playlist-remove"></i>
                    <p>No wish list items shared</p>
                </div>
            `}
        </div>
    `;
    
    container.innerHTML = wishlistHTML;
}

function showPublicShareError() {
    // Hide welcome section and show main content to display error
    const welcomeSection = document.getElementById('welcomeSection');
    const mainContentSection = document.getElementById('mainContentSection');
    if (welcomeSection) welcomeSection.style.display = 'none';
    if (mainContentSection) mainContentSection.style.display = 'block';
    
    // Hide all nav items since this is an invalid/expired share
    const organizersToggle = document.getElementById('organizersToggle');
    if (organizersToggle) {
        const organizersSection = organizersToggle.closest('.nav-section');
        if (organizersSection) {
            organizersSection.style.display = 'none';
        }
    }
    
    // Hide settings, favorites, wishlist
    const settingsButton = document.querySelector('.nav-item[data-page="settings"]');
    if (settingsButton) settingsButton.style.display = 'none';
    
    const favoritesNav = document.querySelector('.nav-item[data-page="favorites"]');
    if (favoritesNav) favoritesNav.style.display = 'none';
    
    const wishlistNav = document.querySelector('.nav-item[data-page="wishlist"]');
    if (wishlistNav) wishlistNav.style.display = 'none';
    
    // Hide admin section if exists
    document.querySelectorAll('.nav-section').forEach(section => {
        const title = section.querySelector('.nav-section-title');
        if (title && title.textContent.includes('Admin')) {
            section.style.display = 'none';
        }
    });
    
    const container = document.getElementById('humidorsContainer');
    if (container) {
        container.innerHTML = `
            <div class="error-container" style="text-align: center; padding: 4rem 2rem; max-width: 600px; margin: 0 auto;">
                <span class="mdi mdi-link-off" style="font-size: 5rem; color: #e74c3c; opacity: 0.9;"></span>
                <h2 style="color: var(--text-primary); margin-top: 1.5rem; margin-bottom: 1rem;">Share Link Unavailable</h2>
                <p style="color: var(--text-secondary); font-size: 1.1rem; line-height: 1.6; margin-bottom: 2rem;">
                    This share link has expired, been revoked, or is no longer valid. 
                    Please contact the person who shared this link with you for a new one.
                </p>
                <a href="/login.html" class="btn btn-primary" style="min-width: 150px;">
                    <span class="mdi mdi-login"></span>
                    Login to Your Account
                </a>
            </div>
        `;
    }
}

// Export functions for global use
window.openHumidorModal = openHumidorModal;
window.openCigarModal = openCigarModal;
window.editHumidor = editHumidor;
window.deleteHumidor = deleteHumidor;
window.editCigar = editCigar;
window.deleteCigar = deleteCigar;
window.toggleFavorite = toggleFavorite;
window.removeFavorite = removeFavorite;
window.openPublicCigarDetails = openPublicCigarDetails;
// Export user management functions for global use
window.showCreateUserModal = showCreateUserModal;
window.editUser = editUser;

window.toggleUserActive = toggleUserActive;
window.showDeleteUserDialog = showDeleteUserDialog;
window.showPasswordResetDialog = showPasswordResetDialog;
window.loadUsers = loadUsers;
// Export share management functions for global use
window.openShareHumidorModal = openShareHumidorModal;
window.updateSharePermission = updateSharePermission;
window.revokeShare = revokeShare;

// ============================================================================
// CIGAR RECOMMENDATION FEATURE
// ============================================================================

// Global variable to store current recommendation
let currentRecommendation = null;

/**
 * Open recommendation modal and fetch a random cigar
 */
async function openRecommendModal() {
    const modal = document.getElementById('recommendModal');
    const modalBody = document.getElementById('recommendModalBody');
    const modalActions = document.getElementById('recommendModalActions');
    
    // Show modal with loading state
    modal.classList.add('active');
    modalBody.innerHTML = `
        <div style="text-align: center; padding: 3rem;">
            <div class="spinner"></div>
            <p style="margin-top: 1rem; color: var(--text-secondary);">
                Finding the perfect cigar for you...
            </p>
        </div>
    `;
    modalActions.style.display = 'none';
    
    try {
        await getRecommendation();
    } catch (error) {
        console.error('Error opening recommendation modal:', error);
        showToast('Failed to get recommendation', 'error');
        closeRecommendModal();
    }
}

/**
 * Fetch a cigar recommendation from the API
 */
async function getRecommendation() {
    const modalBody = document.getElementById('recommendModalBody');
    const modalActions = document.getElementById('recommendModalActions');
    
    try {
        // Determine context - are we viewing a specific humidor?
        let humidorId = null;
        if (currentRoute.humidorId) {
            humidorId = currentRoute.humidorId;
        }
        
        const url = humidorId 
            ? `/api/v1/cigars/recommend?humidor_id=${humidorId}`
            : '/api/v1/cigars/recommend';
        
        const response = await makeAuthenticatedRequest(url);
        
        if (!response.ok) {
            throw new Error('Failed to get recommendation');
        }
        
        const data = await response.json();
        currentRecommendation = data.cigar;
        
        if (data.cigar) {
            // Display the recommended cigar
            modalBody.innerHTML = renderRecommendedCigar(data.cigar, data.message);
            modalActions.style.display = 'flex';
        } else {
            // No cigars available
            modalBody.innerHTML = `
                <div class="recommend-empty-state">
                    <span class="mdi mdi-cigar-off"></span>
                    <p>${escapeHtml(data.message)}</p>
                    <p style="margin-top: 0.5rem; font-size: 0.9rem; color: var(--text-muted);">
                        Add some cigars to your humidors to get recommendations!
                    </p>
                </div>
            `;
            modalActions.style.display = 'none';
        }
    } catch (error) {
        console.error('Error fetching recommendation:', error);
        modalBody.innerHTML = `
            <div class="recommend-empty-state">
                <span class="mdi mdi-alert-circle-outline"></span>
                <p>Failed to load recommendation</p>
            </div>
        `;
        modalActions.style.display = 'none';
        showToast('Failed to get recommendation', 'error');
    }
}

/**
 * Render the recommended cigar details
 */
function renderRecommendedCigar(cigar, message) {
    // Access nested cigar properties
    const cigarData = cigar.cigar || cigar;
    
    // Generate strength indicators
    const strengthScore = cigar.cigar?.strength_score || cigar.strength_score || 0;
    const strengthDots = Array.from({ length: 5 }, (_, i) => 
        `<span class="recommend-strength-dot ${i < strengthScore ? 'filled' : ''}"></span>`
    ).join('');
    
    return `
        <div class="recommend-message">
            ${escapeHtml(message)}
        </div>
        <div class="recommend-cigar-display">
            <div class="recommend-cigar-name">${escapeHtml(cigarData.name)}</div>
            <div class="recommend-cigar-brand">${escapeHtml(cigar.brand_name || 'Unknown Brand')}</div>
            
            <div class="recommend-cigar-details">
                ${cigar.size_name ? `
                    <div class="recommend-detail-item">
                        <span class="recommend-detail-label">Size</span>
                        <span class="recommend-detail-value">${escapeHtml(cigar.size_name)}</span>
                    </div>
                ` : ''}
                
                ${cigar.strength_name ? `
                    <div class="recommend-detail-item">
                        <span class="recommend-detail-label">Strength</span>
                        <span class="recommend-detail-value">
                            ${escapeHtml(cigar.strength_name)}
                            <div class="recommend-strength-indicator">
                                ${strengthDots}
                            </div>
                        </span>
                    </div>
                ` : ''}
                
                ${cigar.origin_name ? `
                    <div class="recommend-detail-item">
                        <span class="recommend-detail-label">Origin</span>
                        <span class="recommend-detail-value">${escapeHtml(cigar.origin_name)}</span>
                    </div>
                ` : ''}
                
                ${cigarData.wrapper ? `
                    <div class="recommend-detail-item">
                        <span class="recommend-detail-label">Wrapper</span>
                        <span class="recommend-detail-value">${escapeHtml(cigarData.wrapper)}</span>
                    </div>
                ` : ''}
                
                <div class="recommend-detail-item">
                    <span class="recommend-detail-label">Quantity Available</span>
                    <span class="recommend-detail-value">${cigarData.quantity}</span>
                </div>
            </div>
            
            ${cigarData.notes ? `
                <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border-color);">
                    <span class="recommend-detail-label">Notes</span>
                    <p style="margin-top: 0.5rem; color: var(--text-secondary); line-height: 1.6;">
                        ${escapeHtml(cigarData.notes)}
                    </p>
                </div>
            ` : ''}
        </div>
    `;
}

/**
 * Get another recommendation (re-roll)
 */
async function getAnotherRecommendation() {
    const modalBody = document.getElementById('recommendModalBody');
    modalBody.innerHTML = `
        <div style="text-align: center; padding: 3rem;">
            <div class="spinner"></div>
            <p style="margin-top: 1rem; color: var(--text-secondary);">
                Finding another option...
            </p>
        </div>
    `;
    
    await getRecommendation();
}

/**
 * Accept the recommendation and decrement quantity
 */
async function acceptRecommendation() {
    if (!currentRecommendation) {
        showToast('No recommendation to accept', 'error');
        return;
    }
    
    try {
        // Access nested cigar data
        const cigarData = currentRecommendation.cigar || currentRecommendation;
        const cigarId = cigarData.id;
        const currentQuantity = cigarData.quantity;
        
        // Check if user has edit permission
        if (currentHumidorPermission === 'view') {
            showToast('You only have view permission for this humidor', 'error');
            return;
        }
        
        const newQuantity = currentQuantity - 1;
        
        // Update cigar quantity
        const response = await makeAuthenticatedRequest(
            `/api/v1/cigars/${cigarId}`,
            {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    quantity: newQuantity
                })
            }
        );
        
        if (!response.ok) {
            throw new Error('Failed to update cigar quantity');
        }
        
        showToast(`Enjoy your ${cigarData.name}! 🔥`, 'success');
        closeRecommendModal();
        
        // Reload humidor to show updated quantity
        if (humidors && humidors.length === 1) {
            await showHumidorDetail(humidors[0].id);
        } else if (currentRoute.humidorId) {
            await showHumidorDetail(currentRoute.humidorId);
        } else {
            await loadHumidors();
        }
        
    } catch (error) {
        console.error('Error accepting recommendation:', error);
        showToast('Failed to update cigar quantity', 'error');
        closeRecommendModal();
    }
}

/**
 * Close the recommendation modal
 */
function closeRecommendModal() {
    const modal = document.getElementById('recommendModal');
    modal.classList.remove('active');
    currentRecommendation = null;
}

/**
 * Escape HTML to prevent XSS
 */
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Export recommendation functions for global use
window.openRecommendModal = openRecommendModal;
window.getRecommendation = getRecommendation;
window.getAnotherRecommendation = getAnotherRecommendation;
window.acceptRecommendation = acceptRecommendation;
window.closeRecommendModal = closeRecommendModal;
