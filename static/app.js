// Application State
let cigars = [];
let filteredCigars = [];
let currentCigar = null;
let isEditing = false;
let currentPage = 'all-cigars';

// Organizer State
let brands = [];
let sizes = [];
let origins = [];
let strengths = [];
let ringGauges = [];
let currentOrganizer = null;
let isEditingOrganizer = false;

// DOM Elements
const elements = {
    loading: document.getElementById('loading'),
    cigarsGrid: document.getElementById('cigarsGrid'),
    emptyState: document.getElementById('emptyState'),
    searchInput: document.getElementById('searchInput'),
    searchBtn: document.getElementById('searchBtn'),
    brandFilter: document.getElementById('brandFilter'),
    strengthFilter: document.getElementById('strengthFilter'),
    originFilter: document.getElementById('originFilter'),
    addCigarBtn: document.getElementById('addCigarBtn'),
    cigarModal: document.getElementById('cigarModal'),
    cigarForm: document.getElementById('cigarForm'),
    closeModal: document.getElementById('closeModal'),
    cancelBtn: document.getElementById('cancelBtn'),
    modalTitle: document.getElementById('modalTitle'),
    saveBtn: document.getElementById('saveBtn'),
    totalCigars: document.getElementById('totalCigars'),
    uniqueBrands: document.getElementById('uniqueBrands'),
    totalValue: document.getElementById('totalValue'),
    toastContainer: document.getElementById('toastContainer')
};

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
        const response = await fetch('/api/v1/cigars', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(cigar)
        });
        if (!response.ok) throw new Error('Failed to create cigar');
        return response.json();
    },

    async updateCigar(id, cigar) {
        const response = await fetch(`/api/v1/cigars/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(cigar)
        });
        if (!response.ok) throw new Error('Failed to update cigar');
        return response.json();
    },

    async deleteCigar(id) {
        const response = await fetch(`/api/v1/cigars/${id}`, {
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
        
        const response = await API.getCigars();
        cigars = response.cigars;
        filteredCigars = [...cigars];
        
        updateStats();
        updateFilters();
        renderCigars();
    } catch (error) {
        console.error('Error loading cigars:', error);
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
    // Load cigars and organizers on page load
    loadCigars();
    loadOrganizers();
    
    // Search and filter event listeners
    elements.searchInput.addEventListener('input', filterCigars);
    elements.searchBtn.addEventListener('click', filterCigars);
    elements.brandFilter.addEventListener('change', filterCigars);
    elements.strengthFilter.addEventListener('change', filterCigars);
    elements.originFilter.addEventListener('change', filterCigars);
    
    // Modal event listeners
    elements.addCigarBtn.addEventListener('click', () => openCigarModal());
    elements.closeModal.addEventListener('click', closeCigarModal);
    elements.cancelBtn.addEventListener('click', closeCigarModal);
    
    // Form submission
    elements.cigarForm.addEventListener('submit', handleFormSubmit);
    
    // Close modal when clicking outside
    elements.cigarModal.addEventListener('click', function(event) {
        if (event.target === elements.cigarModal) {
            closeCigarModal();
        }
    });
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(event) {
        if (event.key === 'Escape') {
            closeCigarModal();
        }
        if (event.key === 'n' && (event.ctrlKey || event.metaKey)) {
            event.preventDefault();
            openCigarModal();
        }
    });
    
    // View toggle functionality
    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.addEventListener('click', function() {
            document.querySelectorAll('.view-btn').forEach(b => b.classList.remove('active'));
            this.classList.add('active');
            
            const view = this.dataset.view;
            elements.cigarsGrid.className = view === 'list' ? 'cigars-list' : 'cigars-grid';
        });
    });

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
    document.getElementById('addCigarCard')?.addEventListener('click', () => openCigarModal());
    document.getElementById('viewBrandsCard')?.addEventListener('click', () => navigateToPage('brands'));
    document.getElementById('humidorManagementCard')?.addEventListener('click', () => navigateToPage('humidors'));

    // Add event listeners to header buttons
    document.getElementById('addCigarBtnHeader')?.addEventListener('click', () => openCigarModal());
    document.getElementById('searchToggle')?.addEventListener('click', () => toggleSearch());
}

function navigateToPage(page) {
    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.toggle('active', item.getAttribute('data-page') === page);
    });

    // Hide all sections
    document.querySelectorAll('.all-cigars-section, .organizer-section').forEach(section => {
        section.style.display = 'none';
    });

    // Show appropriate section and load data
    switch (page) {
        case 'all-cigars':
            document.getElementById('allCigarsSection').style.display = 'block';
            loadCigars();
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

// Global functions for modal opening (called from HTML)
function openBrandModal() { openOrganizerModal('brand'); }
function openSizeModal() { openOrganizerModal('size'); }
function openOriginModal() { openOrganizerModal('origin'); }
function openStrengthModal() { openOrganizerModal('strength'); }
function openRingGaugeModal() { openOrganizerModal('ringGauge'); }

// Initialize navigation when page loads
document.addEventListener('DOMContentLoaded', () => {
    initializeNavigation();
    // Start with all-cigars page
    navigateToPage('all-cigars');
});

// Export functions for global use
window.openCigarModal = openCigarModal;
window.editCigar = editCigar;
window.deleteCigar = deleteCigar;