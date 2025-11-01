// Application State
let humidors = [];
let cigars = [];
let currentHumidor = null;
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

// Search and Filter State
let searchQuery = '';
let selectedBrands = [];
let selectedSizes = [];
let selectedOrigins = [];
let selectedStrengths = [];
let selectedRingGauges = [];
let filteredCigars = [];

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

    const defaultHeaders = {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
    };

    const requestOptions = {
        ...options,
        headers: {
            ...defaultHeaders,
            ...(options.headers || {})
        }
    };

    const response = await fetch(url, requestOptions);
    
    if (response.status === 401) {
        // Token expired or invalid
        localStorage.removeItem('humidor_token');
        localStorage.removeItem('humidor_user');
        window.location.href = '/login.html';
        throw new Error('Authentication failed');
    }
    
    return response;
}

// API Functions
const API = {
    async getCigars(params = {}) {
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
        const response = await fetch('/api/v1/brands');
        if (!response.ok) throw new Error('Failed to fetch brands');
        return response.json();
    },

    async createBrand(brand) {
        const response = await fetch('/api/v1/brands', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(brand)
        });
        if (!response.ok) throw new Error('Failed to create brand');
        return response.json();
    },

    async updateBrand(id, brand) {
        const response = await fetch(`/api/v1/brands/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(brand)
        });
        if (!response.ok) throw new Error('Failed to update brand');
        return response.json();
    },

    async deleteBrand(id) {
        const response = await fetch(`/api/v1/brands/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete brand');
        return response.json();
    },

    // Size API
    async getSizes() {
        const response = await fetch('/api/v1/sizes');
        if (!response.ok) throw new Error('Failed to fetch sizes');
        return response.json();
    },

    async createSize(size) {
        const response = await fetch('/api/v1/sizes', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(size)
        });
        if (!response.ok) throw new Error('Failed to create size');
        return response.json();
    },

    async updateSize(id, size) {
        const response = await fetch(`/api/v1/sizes/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(size)
        });
        if (!response.ok) throw new Error('Failed to update size');
        return response.json();
    },

    async deleteSize(id) {
        const response = await fetch(`/api/v1/sizes/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete size');
        return response.json();
    },

    // Origin API
    async getOrigins() {
        const response = await fetch('/api/v1/origins');
        if (!response.ok) throw new Error('Failed to fetch origins');
        return response.json();
    },

    async createOrigin(origin) {
        const response = await fetch('/api/v1/origins', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(origin)
        });
        if (!response.ok) throw new Error('Failed to create origin');
        return response.json();
    },

    async updateOrigin(id, origin) {
        const response = await fetch(`/api/v1/origins/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(origin)
        });
        if (!response.ok) throw new Error('Failed to update origin');
        return response.json();
    },

    async deleteOrigin(id) {
        const response = await fetch(`/api/v1/origins/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete origin');
        return response.json();
    },

    // Strength API
    async getStrengths() {
        const response = await fetch('/api/v1/strengths');
        if (!response.ok) throw new Error('Failed to fetch strengths');
        return response.json();
    },

    async createStrength(strength) {
        const response = await fetch('/api/v1/strengths', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(strength)
        });
        if (!response.ok) throw new Error('Failed to create strength');
        return response.json();
    },

    async updateStrength(id, strength) {
        const response = await fetch(`/api/v1/strengths/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(strength)
        });
        if (!response.ok) throw new Error('Failed to update strength');
        return response.json();
    },

    async deleteStrength(id) {
        const response = await fetch(`/api/v1/strengths/${id}`, {
            method: 'DELETE'
        });
        if (!response.ok) throw new Error('Failed to delete strength');
        return response.json();
    },

    // Ring Gauge API
    async getRingGauges() {
        const response = await fetch('/api/v1/ring-gauges');
        if (!response.ok) throw new Error('Failed to fetch ring gauges');
        return response.json();
    },

    async createRingGauge(ringGauge) {
        const response = await fetch('/api/v1/ring-gauges', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(ringGauge)
        });
        if (!response.ok) throw new Error('Failed to create ring gauge');
        return response.json();
    },

    async updateRingGauge(id, ringGauge) {
        const response = await fetch(`/api/v1/ring-gauges/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(ringGauge)
        });
        if (!response.ok) throw new Error('Failed to update ring gauge');
        return response.json();
    },

    async deleteRingGauge(id) {
        const response = await fetch(`/api/v1/ring-gauges/${id}`, {
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
    
    const actionButtons = isOutOfStock 
        ? `<button class="action-btn edit-btn" onclick="restockCigar('${cigar.id}')" title="Restock" style="background: #27ae60;">↻</button>`
        : `<button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')">✏️</button>
           <button class="action-btn delete-btn" onclick="deleteCigar('${cigar.id}')">🗑️</button>`;
    
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
    // Update brand filter
    const brands = [...new Set(cigars.map(cigar => cigar.brand))].sort();
    elements.brandFilter.innerHTML = '<option value="">All Brands</option>';
    brands.forEach(brand => {
        const option = document.createElement('option');
        option.value = brand;
        option.textContent = brand;
        elements.brandFilter.appendChild(option);
    });

    // Update origin filter
    const origins = [...new Set(cigars.map(cigar => cigar.origin))].sort();
    elements.originFilter.innerHTML = '<option value="">All Origins</option>';
    origins.forEach(origin => {
        const option = document.createElement('option');
        option.value = origin;
        option.textContent = origin;
        elements.originFilter.appendChild(option);
    });
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
    const brandFilter = elements.brandFilter.value;
    const strengthFilter = elements.strengthFilter.value;
    const originFilter = elements.originFilter.value;

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
    try {
        const cigar = await API.getCigar(id);
        openCigarModal(cigar.humidor_id, cigar);
    } catch (error) {
        console.error('Error fetching cigar:', error);
        showToast('Failed to load cigar details', 'error');
    }
}

async function deleteCigar(id) {
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
        cigars = response.cigars;
        filteredCigars = [...cigars];
        
        updateStats();
        updateFilters();
        renderCigars();
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
        const filled = '█';
        const empty = '░';
        let meter = '';
        for (let i = 1; i <= 5; i++) {
            meter += i <= level ? filled : empty;
        }
        return `<span class="strength-meter" title="Level ${level}/5">${meter}</span>`;
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
    document.getElementById('importUrl').focus();
}

function closeImportUrlModal() {
    const modal = document.getElementById('importUrlModal');
    modal.classList.remove('show');
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
            throw new Error(error.message || 'Failed to scrape URL');
        }
        
        const cigarData = await response.json();
        
        // Populate the cigar form with scraped data
        if (cigarData.brand) document.getElementById('cigarBrand').value = cigarData.brand;
        if (cigarData.name) document.getElementById('cigarName').value = cigarData.name;
        if (cigarData.size) document.getElementById('cigarSize').value = cigarData.size;
        if (cigarData.ring_gauge) document.getElementById('cigarRingGauge').value = cigarData.ring_gauge;
        if (cigarData.strength) document.getElementById('cigarStrength').value = cigarData.strength;
        if (cigarData.origin) document.getElementById('cigarOrigin').value = cigarData.origin;
        
        // Show success message
        statusDiv.innerHTML = '<p class="success-message"><i class="mdi mdi-check-circle"></i> Cigar information imported successfully!</p>';
        
        // Close the import modal after a short delay
        setTimeout(() => {
            closeImportUrlModal();
        }, 1500);
        
    } catch (error) {
        console.error('Import error:', error);
        statusDiv.innerHTML = `<p class="error-message"><i class="mdi mdi-alert-circle"></i> ${error.message}</p>`;
    } finally {
        importBtn.disabled = false;
    }
}

// Event Listeners
document.addEventListener('DOMContentLoaded', function() {
    // Check authentication first
    if (!checkAuth()) {
        window.location.href = '/login.html';
        return;
    }
    
    // Initialize user info display
    initializeUserDisplay();
    
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

    // Initialize navigation and load humidors
    initializeNavigation();
    
    // Event listeners for new interface
    if (elements.createFirstHumidorBtn) {
        elements.createFirstHumidorBtn.addEventListener('click', () => openHumidorModal());
    }
    if (elements.addHumidorBtn) {
        elements.addHumidorBtn.addEventListener('click', () => openHumidorModal());
    }
    if (elements.addHumidorBtnSidebar) {
        elements.addHumidorBtnSidebar.addEventListener('click', () => openHumidorModal());
    }
    if (elements.addHumidorBtnHeader) {
        elements.addHumidorBtnHeader.addEventListener('click', () => openHumidorModal());
    }
    if (elements.addCigarBtn) {
        elements.addCigarBtn.addEventListener('click', () => openCigarModal());
    }
    if (elements.addCigarBtnNav) {
        elements.addCigarBtnNav.addEventListener('click', () => openCigarModal());
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
    
    // Import URL modal events
    const importFromUrlBtn = document.getElementById('importFromUrlBtn');
    const closeImportUrlBtn = document.getElementById('closeImportUrlModal');
    const startImportBtn = document.getElementById('startImportBtn');
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
    document.querySelectorAll('.humidors-section, .organizer-section, .profile-section, .favorites-section').forEach(section => {
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
        welcomeSection.style.display = 'none';
        mainContentSection.style.display = 'block';
        renderHumidorSections();
    }
}

function showWelcomeSection() {
    document.getElementById('welcomeSection').style.display = 'block';
    document.getElementById('mainContentSection').style.display = 'none';
}

// Search and Filter Functions
function applySearchAndFilters() {
    // Start with all cigars
    filteredCigars = [...cigars];
    
    // Apply search query
    if (searchQuery) {
        const query = searchQuery.toLowerCase();
        filteredCigars = filteredCigars.filter(cigar => 
            cigar.brand?.toLowerCase().includes(query) ||
            cigar.name?.toLowerCase().includes(query) ||
            cigar.size?.toLowerCase().includes(query) ||
            cigar.origin?.toLowerCase().includes(query) ||
            cigar.wrapper?.toLowerCase().includes(query) ||
            cigar.notes?.toLowerCase().includes(query)
        );
    }
    
    // Apply brand filter
    if (selectedBrands.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedBrands.includes(cigar.brand)
        );
    }
    
    // Apply size filter
    if (selectedSizes.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedSizes.includes(cigar.size)
        );
    }
    
    // Apply origin filter
    if (selectedOrigins.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedOrigins.includes(cigar.origin)
        );
    }
    
    // Apply strength filter
    if (selectedStrengths.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedStrengths.includes(cigar.strength)
        );
    }
    
    // Apply ring gauge filter
    if (selectedRingGauges.length > 0) {
        filteredCigars = filteredCigars.filter(cigar => 
            selectedRingGauges.includes(cigar.ring_gauge)
        );
    }
    
    updateFilterBadges();
    renderHumidorSections();
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
}

function clearFilters() {
    searchQuery = '';
    selectedBrands = [];
    selectedSizes = [];
    selectedOrigins = [];
    selectedStrengths = [];
    selectedRingGauges = [];
    document.getElementById('cigarSearchInput').value = '';
    applySearchAndFilters();
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
    
    // Get unique values for the current filter type from all cigars
    let uniqueValues = [];
    switch(filterType) {
        case 'brand':
            uniqueValues = [...new Set(cigars.map(c => c.brand).filter(Boolean))].sort();
            tempSelectedItems = [...selectedBrands];
            break;
        case 'size':
            uniqueValues = [...new Set(cigars.map(c => c.size).filter(Boolean))].sort();
            tempSelectedItems = [...selectedSizes];
            break;
        case 'origin':
            uniqueValues = [...new Set(cigars.map(c => c.origin).filter(Boolean))].sort();
            tempSelectedItems = [...selectedOrigins];
            break;
        case 'strength':
            uniqueValues = [...new Set(cigars.map(c => c.strength).filter(Boolean))].sort();
            tempSelectedItems = [...selectedStrengths];
            break;
        case 'ringGauge':
            uniqueValues = [...new Set(cigars.map(c => c.ring_gauge).filter(Boolean))].sort();
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
    // Use helper functions to resolve IDs to names
    const brandName = getBrandName(cigar.brand_id);
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
                ${quantityControls}
            </div>
        </div>
    `;
}

function openHumidorModal(humidor = null) {
    isEditingHumidor = !!humidor;
    currentHumidor = humidor;
    
    const modal = document.getElementById('humidorModal');
    const title = document.getElementById('humidorModalTitle');
    const form = document.getElementById('humidorForm');
    
    title.textContent = isEditingHumidor ? 'Edit Humidor' : 'Add New Humidor';
    
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
    const cigar = cigars.find(c => c.id === cigarId);
    if (!cigar) {
        console.error('Cigar not found:', cigarId);
        return;
    }
    
    const modal = document.getElementById('reportCardModal');
    
    // Set image
    const image = document.getElementById('reportCardImage');
    image.src = cigar.image_url || '/static/cigar-placeholder.png';
    
    // Set brand and name
    document.getElementById('reportCardBrand').textContent = getBrandName(cigar.brand_id);
    document.getElementById('reportCardName').textContent = cigar.name;
    
    // Set details
    const humidor = humidors.find(h => h.id === cigar.humidor_id);
    document.getElementById('reportCardHumidor').textContent = humidor ? humidor.name : '-';
    document.getElementById('reportCardQuantity').textContent = cigar.quantity || '-';
    document.getElementById('reportCardSize').textContent = getSizeName(cigar.size_id);
    document.getElementById('reportCardRingGauge').textContent = getRingGaugeName(cigar.ring_gauge_id);
    document.getElementById('reportCardStrength').textContent = getStrengthName(cigar.strength_id);
    document.getElementById('reportCardOrigin').textContent = getOriginName(cigar.origin_id);
    document.getElementById('reportCardPrice').textContent = cigar.price ? `$${parseFloat(cigar.price).toFixed(2)}` : '-';
    
    // Format purchase date
    if (cigar.purchase_date) {
        const date = new Date(cigar.purchase_date);
        document.getElementById('reportCardPurchaseDate').textContent = date.toLocaleDateString();
    } else {
        document.getElementById('reportCardPurchaseDate').textContent = '-';
    }
    
    document.getElementById('reportCardNotes').textContent = cigar.notes || 'No notes available';
    
    // Set up action buttons
    const editBtn = document.getElementById('reportCardEditBtn');
    const deleteBtn = document.getElementById('reportCardDeleteBtn');
    
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
    humidors.forEach(humidor => {
        const option = document.createElement('option');
        option.value = humidor.id;
        option.textContent = humidor.name;
        if (humidorId && humidor.id === humidorId) {
            option.selected = true;
        }
        humidorSelect.appendChild(option);
    });
    
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
        document.getElementById('cigarNotes').value = cigar.notes || '';
        document.getElementById('cigarImageUrl').value = cigar.image_url || '';
        
        if (cigar.purchase_date) {
            document.getElementById('cigarPurchaseDate').value = cigar.purchase_date.split('T')[0];
        }
        
        // Set organizer dropdowns using IDs
        if (cigar.humidor_id) humidorSelect.value = cigar.humidor_id;
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
}

async function saveHumidor() {
    const form = document.getElementById('humidorForm');
    const formData = new FormData(form);
    
    const humidorData = {
        name: formData.get('name'),
        type: formData.get('type'),
        capacity: parseInt(formData.get('capacity')),
        location: formData.get('location') || null,
        description: formData.get('description') || null
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
        showToast('Failed to save humidor', 'error');
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
    
    const cigarData = {
        humidor_id: formData.get('humidor_id') || null,
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
        notes: formData.get('notes') || null,
        image_url: imageUrl
    };

    console.log('=== saveCigar() called ===');
    console.log('→ Form data extracted:', cigarData);
    
    // Validate that a humidor is selected
    if (!cigarData.humidor_id || cigarData.humidor_id.trim() === '') {
        console.error('✗ No humidor selected!');
        showToast('Please select a humidor for this cigar', 'error');
        return;
    }
    
    console.log(`✓ Humidor selected: ${cigarData.humidor_id}`);

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
            console.log('→ Reloading humidors and cigars...');
            
            // Clear the file input
            document.getElementById('cigarImageUpload').value = '';
            
            await loadHumidors();
            showToast(isEditingCigar ? 'Cigar updated successfully!' : 'Cigar added successfully!', 'success');
            closeCigarModal();
        } else {
            const errorData = await response.json();
            console.error('✗ Failed to save cigar:', errorData);
            showToast(errorData.error || 'Failed to save cigar', 'error');
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

async function deleteCigar(id, skipConfirm = false) {
    // Only show confirmation if called manually (not from quantity update)
    if (!skipConfirm && !confirm('Mark this cigar as out of stock? It will remain in your inventory and can be restocked later.')) return;
    
    try {
        const response = await makeAuthenticatedRequest(`/api/v1/cigars/${id}`, {
            method: 'DELETE'
        });
        
        if (response && response.ok) {
            showToast('Cigar marked as out of stock');
            await loadHumidors();
        } else {
            throw new Error('Failed to mark cigar as out of stock');
        }
    } catch (error) {
        console.error('Error marking cigar as out of stock:', error);
        showToast('Failed to mark cigar as out of stock', 'error');
    }
}

// Restock cigar function
async function restockCigar(id) {
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
    
    // If quantity reaches 0, mark as out of stock by calling the backend DELETE endpoint
    if (newQuantity === 0) {
        if (!confirm('Mark this cigar as out of stock? You can restock it later.')) {
            return;
        }
        
        try {
            const response = await makeAuthenticatedRequest(`/api/v1/cigars/${cigarId}`, {
                method: 'DELETE'
            });
            
            if (response && response.ok) {
                showToast('Cigar marked as out of stock');
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
    
    const imageHtml = cigar.image_url 
        ? `<img src="${cigar.image_url}" alt="${cigar.name}" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
           <img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="display: none; width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`
        : `<img src="/static/cigar-placeholder.png" alt="Cigar placeholder" style="width: 100%; height: 100%; object-fit: contain; padding: 2rem;">`;
    
    // For out of stock cigars, disable click to open report card and show different actions
    const cardOnClick = cigar.out_of_stock ? '' : `onclick="openReportCard('${cigar.id}')"`;
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
        <div class="cigar-card" data-cigar-id="${cigar.id}" ${cardOnClick} ${cardStyle}>
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
            </div>
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

// Global functions for modal opening (called from HTML)
function openBrandModal() { openOrganizerModal('brand'); }
function openSizeModal() { openOrganizerModal('size'); }
function openOriginModal() { openOrganizerModal('origin'); }
function openStrengthModal() { openOrganizerModal('strength'); }
function openRingGaugeModal() { openOrganizerModal('ringGauge'); }

// Export functions for global use
window.openHumidorModal = openHumidorModal;
window.openCigarModal = openCigarModal;
window.editHumidor = editHumidor;
window.deleteHumidor = deleteHumidor;
window.editCigar = editCigar;
window.deleteCigar = deleteCigar;
window.toggleFavorite = toggleFavorite;
window.removeFavorite = removeFavorite;