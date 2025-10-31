pub mod auth;
pub mod cigars;
pub mod humidors;
pub mod brands;
pub mod sizes;
pub mod origins;
pub mod strengths;
pub mod ring_gauges;
pub mod favorites;

// Re-export handler functions with specific names to avoid conflicts
pub use auth::{get_setup_status, create_setup_user, login_user, get_current_user, update_current_user, change_password};
pub use cigars::{get_cigars, get_cigar, create_cigar, update_cigar, delete_cigar, scrape_cigar_url};

pub use brands::{
    get_brands, 
    create_brand, 
    update_brand, 
    delete_brand
};

pub use sizes::{
    get_sizes, 
    create_size, 
    update_size, 
    delete_size
};

pub use origins::{
    get_origins, 
    create_origin, 
    update_origin, 
    delete_origin
};

pub use strengths::{
    get_strengths, 
    create_strength, 
    update_strength, 
    delete_strength
};

pub use ring_gauges::{
    get_ring_gauges, 
    create_ring_gauge, 
    update_ring_gauge, 
    delete_ring_gauge
};

pub use humidors::{
    get_humidors,
    get_humidor,
    create_humidor,
    update_humidor,
    delete_humidor,
    get_humidor_cigars
};

pub use favorites::{
    get_favorites,
    add_favorite,
    remove_favorite,
    is_favorite
};