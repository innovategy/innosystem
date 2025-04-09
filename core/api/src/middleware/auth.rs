use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use axum::body::Body;
use uuid::Uuid;
use tracing::{debug, error, info};

use crate::state::AppState;

// Define the authorization roles
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Reseller,
    Customer,
}

// Admin user representation
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: String,
}

// Reseller user representation
#[derive(Debug, Clone)]
pub struct ResellerUser {
    pub id: Uuid,
    pub name: String,
}

// Customer user representation
#[derive(Debug, Clone)]
pub struct CustomerUser {
    pub id: Uuid,
    pub name: String,
    pub reseller_id: Option<Uuid>,
}

// API authentication middleware for admin access
pub async fn admin_auth<B>(
    State(app_state): State<AppState>,
    mut req: Request<B>,
    next: Next,
) -> Result<Response, StatusCode> 
where
    B: Send + 'static,
{
    debug!("Processing admin authentication");
    
    // Get the API key from the header
    let api_key = get_api_key_from_header(&req)
        .ok_or_else(|| {
            error!("Missing API key for admin authentication");
            StatusCode::UNAUTHORIZED
        })?;
    
    // For now, the admin API key is hardcoded or retrieved from configuration
    // In a real-world scenario, this would be securely stored and compared
    if api_key == app_state.config.admin_api_key {
        // Admin is authenticated
        info!("Admin authentication successful");
        let admin = AdminUser {
            id: "admin".to_string(),
        };
        
        // Add the admin user to the request extensions
        req.extensions_mut().insert(admin);
        
        // Continue to the handler
        // Convert request body type to Body for compatibility with next.run()
        let (parts, _) = req.into_parts();
        let req = Request::from_parts(parts, Body::empty());
        Ok(next.run(req).await)
    } else {
        error!("Invalid admin API key");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// API authentication middleware for reseller access
pub async fn reseller_auth<B>(
    State(app_state): State<AppState>,
    mut req: Request<B>,
    next: Next,
) -> Result<Response, StatusCode> 
where
    B: Send + 'static,
{
    debug!("Processing reseller authentication");
    
    // Get the API key from the header
    let api_key = get_api_key_from_header(&req)
        .ok_or_else(|| {
            error!("Missing API key for reseller authentication");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Check if this is an admin key first (admins can access reseller endpoints)
    if api_key == app_state.config.admin_api_key {
        let admin = AdminUser {
            id: "admin".to_string(),
        };
        req.extensions_mut().insert(admin);
        
        // Convert request body type to Body for compatibility with next.run()
        let (parts, _) = req.into_parts();
        let req = Request::from_parts(parts, Body::empty());
        return Ok(next.run(req).await);
    }
    
    // TODO: Update once ResellerRepository is implemented in Phase 3.3.2
    // For now, we'll use a stub implementation which just returns unauthorized
    error!("Reseller repository not yet implemented");
    return Err(StatusCode::UNAUTHORIZED);
    
    // Note: The code below is unreachable until ResellerRepository is implemented
    // It's kept here as a template for the future implementation
}

// API authentication middleware for customer access
pub async fn customer_auth<B>(
    State(app_state): State<AppState>,
    mut req: Request<B>,
    next: Next,
) -> Result<Response, StatusCode> 
where
    B: Send + 'static,
{
    debug!("Processing customer authentication");
    
    // Get the API key from the header
    let api_key = get_api_key_from_header(&req)
        .ok_or_else(|| {
            error!("Missing API key for customer authentication");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Check if this is an admin key first (admins can access customer endpoints)
    if api_key == app_state.config.admin_api_key {
        let admin = AdminUser {
            id: "admin".to_string(),
        };
        req.extensions_mut().insert(admin);
        
        // Convert request body type to Body for compatibility with next.run()
        let (parts, _) = req.into_parts();
        let req = Request::from_parts(parts, Body::empty());
        return Ok(next.run(req).await);
    }
    
    // Look up the reseller by API key to check if this is a reseller
    // TODO: Update once ResellerRepository is implemented in Phase 3.3.2
    // For now, we'll bypass this check and assume it's not a reseller
    // Just continue with customer authentication
    
    // Look up the customer by API key
    let customer = match app_state.customer_repo.find_by_api_key(&api_key).await {
        Ok(customer) => customer,
        Err(e) => {
            error!("Failed to find customer with API key: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Note: Customer struct doesn't have an 'active' field in the current implementation
    // For now, we'll assume all customers are active
    // TODO: Add active field to Customer model in Phase 3.3.2
    
    // Customer is authenticated
    info!("Customer authentication successful: {}", customer.id);
    let customer_user = CustomerUser {
        id: customer.id,
        name: customer.name,
        reseller_id: customer.reseller_id,
    };
    
    // Add the customer user to the request extensions
    req.extensions_mut().insert(customer_user);
    
    // Continue to the handler
    // Convert request body type to Body for compatibility with next.run()
    let (parts, _) = req.into_parts();
    let req = Request::from_parts(parts, Body::empty());
    Ok(next.run(req).await)
}

// Helper function to get the API key from the request header
fn get_api_key_from_header<B>(req: &Request<B>) -> Option<String> {
    // First try the Authorization header with Bearer scheme
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_value) = auth_header.to_str() {
            if auth_value.starts_with("Bearer ") {
                return Some(auth_value[7..].to_string());
            }
        }
    }
    
    // Then try the X-API-Key header
    if let Some(api_key_header) = req.headers().get("X-API-Key") {
        if let Ok(api_key) = api_key_header.to_str() {
            return Some(api_key.to_string());
        }
    }
    
    None
}

// Utility function to verify access to a specific customer's resources
pub async fn verify_customer_access(
    customer_id: Uuid,
    extension: &Extension<Option<AdminUser>>,
    extension_reseller: &Extension<Option<ResellerUser>>,
    extension_customer: &Extension<Option<CustomerUser>>,
) -> Result<(), StatusCode> {
    // Admins have access to all customer resources
    if extension.0.is_some() {
        return Ok(());
    }
    
    // Check if the authenticated user is a reseller
    if let Some(_reseller) = &extension_reseller.0 {
        // The reseller repository would be used to check if this customer belongs to this reseller
        // For simplicity, we'll implement this check later
        // For now, just grant access to resellers
        return Ok(());
    }
    
    // Check if the authenticated user is the customer
    if let Some(customer) = &extension_customer.0 {
        if customer.id == customer_id {
            return Ok(());
        }
    }
    
    // Access denied
    Err(StatusCode::FORBIDDEN)
}
