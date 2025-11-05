pub mod auth;
pub mod brands;
pub mod cigars;
pub mod favorites;
pub mod humidors;
pub mod origins;
pub mod ring_gauges;
pub mod sizes;
pub mod strengths;
pub mod wish_list;

// Re-export handler functions with specific names to avoid conflicts
pub use auth::{
    change_password, check_email_config, create_setup_user, forgot_password, get_current_user, 
    get_setup_status, login_user, reset_password, update_current_user,
};
pub use cigars::{
    create_cigar, delete_cigar, get_cigar, get_cigars, scrape_cigar_url, update_cigar,
};

pub use brands::{create_brand, delete_brand, get_brands, update_brand};

pub use sizes::{create_size, delete_size, get_sizes, update_size};

pub use origins::{create_origin, delete_origin, get_origins, update_origin};

pub use strengths::{create_strength, delete_strength, get_strengths, update_strength};

pub use ring_gauges::{create_ring_gauge, delete_ring_gauge, get_ring_gauges, update_ring_gauge};

pub use humidors::{
    create_humidor, delete_humidor, get_humidor, get_humidor_cigars, get_humidors, update_humidor,
};

pub use favorites::{add_favorite, get_favorites, is_favorite, remove_favorite};

pub use wish_list::{
    add_to_wish_list, check_wish_list, get_wish_list, remove_from_wish_list,
    update_wish_list_notes,
};
