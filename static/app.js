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
        
        if (userInfo && userName) {
            userName.textContent = currentUser.full_name || currentUser.username;
            userInfo.style.display = 'flex';
        }
    } else {
        // Hide user info if no user
        const userInfo = document.getElementById('userInfo');
        if (userInfo) {
            userInfo.style.display = 'none';
        }
    }
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
        const response = await fetch(`/api/v1/cigars/${id}`);
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

// Utility Functions
function showToast(message, type = 'success') {
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.innerHTML = `<div class="toast-message">${message}</div>`;
    
    elements.toastContainer.appendChild(toast);
    
    setTimeout(() => {
        toast.remove();
    }, 5000);
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
    card.innerHTML = `
        <div class="cigar-card-image">
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
                <div class="cigar-brand">${cigar.brand}</div>
                <div class="cigar-actions">
                    <button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteCigar('${cigar.id}')">🗑️</button>
                </div>
            </div>
            
            <div class="cigar-name">${cigar.name}</div>
            
            <div class="cigar-details">
                <div class="detail-item">
                    <div class="detail-label">Size</div>
                    <div class="detail-value">${cigar.size}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Strength</div>
                    <div class="detail-value" style="color: ${getStrengthColor(cigar.strength)}">${cigar.strength}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Origin</div>
                    <div class="detail-value">${cigar.origin}</div>
                </div>
                <div class="detail-item">
                    <div class="detail-label">Location</div>
                    <div class="detail-value">${cigar.humidor_location || 'Not specified'}</div>
                </div>
            </div>
            
            <div class="cigar-footer">
                <div class="quantity-badge">${cigar.quantity} left</div>
                <div class="price-tag">${formatPrice(cigar.price)}</div>
            </div>
        </div>
    `;
    
    return card;
}

function updateStats() {
    const totalQuantity = cigars.reduce((sum, cigar) => sum + cigar.quantity, 0);
    const uniqueBrands = new Set(cigars.map(cigar => cigar.brand)).size;
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
            cigar.brand.toLowerCase().includes(searchTerm) ||
            cigar.name.toLowerCase().includes(searchTerm) ||
            (cigar.notes && cigar.notes.toLowerCase().includes(searchTerm));
        
        const matchesBrand = !brandFilter || cigar.brand === brandFilter;
        const matchesStrength = !strengthFilter || cigar.strength === strengthFilter;
        const matchesOrigin = !originFilter || cigar.origin === originFilter;

        return matchesSearch && matchesBrand && matchesStrength && matchesOrigin;
    });

    renderCigars();
}

function openCigarModal(cigar = null) {
    isEditing = !!cigar;
    currentCigar = cigar;
    
    elements.modalTitle.textContent = isEditing ? 'Edit Cigar' : 'Add New Cigar';
    elements.saveBtn.textContent = isEditing ? 'Update Cigar' : 'Save Cigar';
    
    // Reset form
    elements.cigarForm.reset();
    
    // Populate form if editing
    if (isEditing && cigar) {
        Object.keys(cigar).forEach(key => {
            const input = document.getElementById(key);
            if (input) {
                if (key === 'purchase_date' && cigar[key]) {
                    input.value = cigar[key].split('T')[0];
                } else {
                    input.value = cigar[key] || '';
                }
            }
        });
    }
    
    elements.cigarModal.classList.add('show');
    document.body.style.overflow = 'hidden';
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
        openCigarModal(cigar);
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
        [brands, sizes, origins, strengths, ringGauges] = await Promise.all([
            OrganizerAPI.getBrands(),
            OrganizerAPI.getSizes(),
            OrganizerAPI.getOrigins(),
            OrganizerAPI.getStrengths(),
            OrganizerAPI.getRingGauges()
        ]);
        
        // Update navigation counts
        updateOrganizerCounts();
        
        // Update form dropdowns
        updateFormDropdowns();
    } catch (error) {
        console.error('Error loading organizers:', error);
        showToast('Failed to load organizers', 'error');
    }
}

function updateOrganizerCounts() {
    document.getElementById('brandCount').textContent = brands.length;
    document.getElementById('sizeCount').textContent = sizes.length;
    document.getElementById('originCount').textContent = origins.length;
    document.getElementById('strengthCount').textContent = strengths.length;
    document.getElementById('ringGaugeCount').textContent = ringGauges.length;
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
            'strengths': '💪',
            'ringGauges': '⭕'
        };
        return icons[type] || '📋';
    };

    const getDisplayValue = (organizer, type) => {
        switch(type) {
            case 'brands':
                return organizer.country ? `${organizer.name} (${organizer.country})` : organizer.name;
            case 'sizes':
                const details = [];
                if (organizer.length_inches) details.push(`${organizer.length_inches}"`);
                if (organizer.ring_gauge) details.push(`RG ${organizer.ring_gauge}`);
                return details.length ? `${organizer.name} (${details.join(', ')})` : organizer.name;
            case 'origins':
                return organizer.region ? `${organizer.name}, ${organizer.region}` : `${organizer.name}, ${organizer.country}`;
            case 'strengths':
                return `${organizer.name} (Level ${organizer.level})`;
            case 'ringGauges':
                const names = organizer.common_names && organizer.common_names.length > 0 
                    ? ` (${organizer.common_names.join(', ')})` 
                    : '';
                return `${organizer.gauge}${names}`;
            default:
                return organizer.name || organizer.gauge;
        }
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

    modal.style.display = 'flex';
}

function populateOrganizerForm(type, organizer) {
    const form = document.getElementById(`${type}Form`);
    if (!form || !organizer) return;

    // Common fields
    const nameField = form.querySelector('[name="name"]');
    if (nameField && organizer.name) {
        nameField.value = organizer.name;
    }

    const descField = form.querySelector('[name="description"]');
    if (descField && organizer.description) {
        descField.value = organizer.description;
    }

    // Type-specific fields
    switch(type) {
        case 'brand':
            if (organizer.country) form.querySelector('[name="country"]').value = organizer.country;
            if (organizer.website) form.querySelector('[name="website"]').value = organizer.website;
            break;
        case 'size':
            if (organizer.length_inches) form.querySelector('[name="length_inches"]').value = organizer.length_inches;
            if (organizer.ring_gauge) form.querySelector('[name="ring_gauge"]').value = organizer.ring_gauge;
            break;
        case 'origin':
            if (organizer.country) form.querySelector('[name="country"]').value = organizer.country;
            if (organizer.region) form.querySelector('[name="region"]').value = organizer.region;
            break;
        case 'strength':
            if (organizer.level) form.querySelector('[name="level"]').value = organizer.level;
            break;
        case 'ringGauge':
            if (organizer.gauge) form.querySelector('[name="gauge"]').value = organizer.gauge;
            if (organizer.common_names) {
                form.querySelector('[name="common_names"]').value = organizer.common_names.join(', ');
            }
            break;
    }
}

function closeOrganizerModal(type) {
    const modal = document.getElementById(`${type}Modal`);
    if (modal) {
        modal.style.display = 'none';
        isEditingOrganizer = false;
        currentOrganizer = null;
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
        addCigarBtn: document.getElementById('addCigarBtn'),
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
    if (elements.addCigarBtn) {
        elements.addCigarBtn.addEventListener('click', () => openCigarModal());
    }
    
    // Logout button event
    const logoutBtn = document.getElementById('logoutBtn');
    if (logoutBtn) {
        logoutBtn.addEventListener('click', (e) => {
            e.preventDefault();
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
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(event) {
        if (event.key === 'Escape') {
            closeHumidorModal();
            closeCigarModal();
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
    
    // Initialize the interface
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
}

function navigateToPage(page) {
    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.toggle('active', item.getAttribute('data-page') === page);
    });

    // Hide all sections
    document.querySelectorAll('.humidors-section, .organizer-section').forEach(section => {
        section.style.display = 'none';
    });

    // Show appropriate section and load data
    switch (page) {
        case 'humidors':
            document.getElementById('humidorsSection').style.display = 'block';
            loadHumidors();
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
        const response = await makeAuthenticatedRequest('/api/v1/humidors');

        if (response.ok) {
            humidors = await response.json();
            // Load cigars for each humidor
            cigars = [];
            for (const humidor of humidors) {
                const cigarResponse = await makeAuthenticatedRequest(`/api/v1/humidors/${humidor.id}/cigars`);
                if (cigarResponse.ok) {
                    const humidorCigars = await cigarResponse.json();
                    cigars.push(...humidorCigars);
                }
            }
        } else {
            console.error('Failed to load humidors:', response.status, response.statusText);
            humidors = [];
            cigars = [];
        }
        
        // Show appropriate section based on whether humidors exist
        showAppropriateSection();
    } catch (error) {
        console.error('Error loading humidors:', error);
        if (error.message !== 'Authentication failed') {
            showToast('Failed to load humidors', 'error');
        }
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
        updateCollectionSummary();
        renderHumidorSections();
    }
}

function showWelcomeSection() {
    document.getElementById('welcomeSection').style.display = 'block';
    document.getElementById('mainContentSection').style.display = 'none';
}

function updateCollectionSummary() {
    const totalCigars = cigars.length;
    const summaryText = `${humidors.length} humidor${humidors.length !== 1 ? 's' : ''} • ${totalCigars} cigar${totalCigars !== 1 ? 's' : ''}`;
    document.getElementById('collectionSummary').textContent = summaryText;
}

function renderHumidorSections() {
    const container = document.getElementById('humidorsContainer');
    
    if (humidors.length === 0) {
        container.innerHTML = '';
        return;
    }
    
    container.innerHTML = humidors.map(humidor => {
        const humidorCigars = cigars.filter(cigar => cigar.humidor_id === humidor.id);
        return createHumidorSection(humidor, humidorCigars);
    }).join('');
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
                    <button class="action-btn add-cigar-btn" onclick="openCigarModal('${humidor.id}')" title="Add Cigar">
                        <span class="add-icon">+</span>
                        Add Cigar
                    </button>
                    <button class="action-btn edit-btn" onclick="editHumidor('${humidor.id}')" title="Edit Humidor">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteHumidor('${humidor.id}')" title="Delete Humidor">🗑️</button>
                </div>
            </div>
            
            <div class="cigars-grid" data-humidor-id="${humidor.id}">
                ${humidorCigars.length > 0 
                    ? humidorCigars.map(cigar => createCigarCard(cigar)).join('') 
                    : '<div class="empty-cigars-message">No cigars in this humidor yet. <button class="link-btn" onclick="openCigarModal(\'' + humidor.id + '\')">Add your first cigar</button></div>'
                }
            </div>
        </div>
    `;
}

function createCigarCard(cigar) {
    return `
        <div class="cigar-card" data-cigar-id="${cigar.id}">
            <div class="cigar-card-header">
                <div class="cigar-info">
                    <h4 class="cigar-brand">${cigar.brand}</h4>
                    <h3 class="cigar-name">${cigar.name}</h3>
                </div>
                <div class="cigar-actions">
                    <button class="action-btn edit-btn" onclick="editCigar('${cigar.id}')" title="Edit">✏️</button>
                    <button class="action-btn delete-btn" onclick="deleteCigar('${cigar.id}')" title="Delete">🗑️</button>
                </div>
            </div>
            
            <div class="cigar-details">
                <div class="detail-row">
                    <span class="detail-label">Size:</span>
                    <span class="detail-value">${cigar.size}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Strength:</span>
                    <span class="detail-value strength-${cigar.strength?.toLowerCase()}">${cigar.strength}</span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Origin:</span>
                    <span class="detail-value">${cigar.origin}</span>
                </div>
                ${cigar.price ? `<div class="detail-row">
                    <span class="detail-label">Price:</span>
                    <span class="detail-value">$${parseFloat(cigar.price).toFixed(2)}</span>
                </div>` : ''}
            </div>
            
            <div class="cigar-footer">
                <span class="quantity-badge">${cigar.quantity} ${cigar.quantity === 1 ? 'cigar' : 'cigars'}</span>
                ${cigar.purchase_date ? `<span class="purchase-date">Purchased: ${new Date(cigar.purchase_date).toLocaleDateString()}</span>` : ''}
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
    
    modal.style.display = 'flex';
}

function closeHumidorModal() {
    document.getElementById('humidorModal').style.display = 'none';
    isEditingHumidor = false;
    currentHumidor = null;
}

function openCigarModal(humidorId = null, cigar = null) {
    isEditingCigar = !!cigar;
    currentCigar = cigar;
    
    const modal = document.getElementById('cigarModal');
    const title = document.getElementById('cigarModalTitle');
    const form = document.getElementById('cigarForm');
    const humidorSelect = document.getElementById('cigarHumidor');
    
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
    
    if (isEditingCigar) {
        // Populate form with cigar data
        Object.keys(cigar).forEach(key => {
            const input = document.getElementById(`cigar${key.charAt(0).toUpperCase() + key.slice(1)}`);
            if (input) {
                if (key === 'purchase_date' && cigar[key]) {
                    input.value = cigar[key].split('T')[0];
                } else {
                    input.value = cigar[key] || '';
                }
            }
        });
    } else {
        form.reset();
        if (humidorId) {
            humidorSelect.value = humidorId;
        }
    }
    
    modal.style.display = 'flex';
}

function closeCigarModal() {
    document.getElementById('cigarModal').style.display = 'none';
    isEditingCigar = false;
    currentCigar = null;
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
        updateCollectionSummary();
        showToast('Humidor updated successfully!');
        closeHumidorModal();
    }
}

function deleteHumidor(id) {
    if (!confirm('Are you sure you want to delete this humidor and all its cigars?')) return;
    
    // Remove cigars from this humidor
    cigars = cigars.filter(cigar => cigar.humidor_id !== id);
    
    // Remove the humidor
    humidors = humidors.filter(h => h.id !== id);
    
    showAppropriateSection();
    showToast('Humidor deleted successfully!');
}

function createCigar(cigarData) {
    const newCigar = {
        id: Date.now().toString(),
        ...cigarData,
        created_at: new Date().toISOString()
    };
    
    cigars.push(newCigar);
    renderHumidorSections();
    updateCollectionSummary();
    showToast('Cigar added successfully!');
    closeCigarModal();
}

function editCigar(id) {
    const cigar = cigars.find(c => c.id === id);
    if (cigar) openCigarModal(null, cigar);
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
        updateCollectionSummary();
        showToast('Cigar updated successfully!');
        closeCigarModal();
    }
}

function deleteCigar(id) {
    if (!confirm('Are you sure you want to delete this cigar?')) return;
    
    cigars = cigars.filter(c => c.id !== id);
    renderHumidorSections();
    updateCollectionSummary();
    showToast('Cigar deleted successfully!');
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