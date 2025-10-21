// Application State
let cigars = [];
let filteredCigars = [];
let currentCigar = null;
let isEditing = false;
let currentPage = 'all-cigars';

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
    
    for (const [key, value] = formData.entries()) {
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

// Event Listeners
document.addEventListener('DOMContentLoaded', function() {
    // Load cigars on page load
    loadCigars();
    
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
});

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
    document.getElementById('allCigarsSection')?.style.setProperty('display', 'none');

    // Update page title and show appropriate section
    const pageTitle = document.getElementById('pageTitle');
    const pageSubtitle = document.getElementById('pageSubtitle');

    switch (page) {
        case 'all-cigars':
            pageTitle.textContent = 'All Cigars';
            pageSubtitle.textContent = 'Complete collection view';
            document.getElementById('allCigarsSection')?.style.setProperty('display', 'block');
            loadCigars();
            break;
        case 'brands':
            pageTitle.textContent = 'Brands';
            pageSubtitle.textContent = 'Organize by brand';
            showBrandsView();
            break;
        case 'humidors':
            pageTitle.textContent = 'Humidors';
            pageSubtitle.textContent = 'Manage your humidor locations';
            showHumidorsView();
            break;
    }

    currentPage = page;
}

function showBrandsView() {
    const pageContent = document.querySelector('.page-content');
    pageContent.innerHTML = `
        <div class="brands-section">
            <h3>Brands in Your Collection</h3>
            <div class="brands-grid">
                <p>Brands view coming soon...</p>
            </div>
        </div>
    `;
}

function showHumidorsView() {
    const pageContent = document.querySelector('.page-content');
    pageContent.innerHTML = `
        <div class="humidors-section">
            <h3>Your Humidors</h3>
            <div class="humidors-grid">
                <p>Humidors management coming soon...</p>
            </div>
        </div>
    `;
}

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