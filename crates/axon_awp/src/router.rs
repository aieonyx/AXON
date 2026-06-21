// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P66 — AWP router: register sovereign endpoints, resolve requests

use std::collections::HashMap;
use crate::types::{AwpAddress, AwpRequest, AwpResponse, AwpError};
use crate::parser::parse;

/// Handler function type: takes a request, returns a response
pub type HandlerFn = Box<dyn Fn(&AwpRequest) -> AwpResponse + Send + Sync>;

/// Route key: "name.category" or "name.category.region"
fn route_key(addr: &AwpAddress) -> String {
    match &addr.region {
        Some(r) => format!("{}.{}.{}", addr.name, addr.category, r),
        None    => format!("{}.{}", addr.name, addr.category),
    }
}

/// AWP sovereign router
pub struct AwpRouter {
    routes: HashMap<String, HandlerFn>,
}

impl AwpRouter {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    /// Register an endpoint. Key: "name.category" or "name.category.region"
    pub fn register(&mut self, address: &str, handler: HandlerFn) -> Result<(), AwpError> {
        let uri = format!("awp://{}", address);
        let addr = parse(&uri)?;
        let key = route_key(&addr);
        self.routes.insert(key, handler);
        Ok(())
    }

    /// Resolve and dispatch a request. Returns sovereign 404 if no route matches.
    pub fn dispatch(&self, req: &AwpRequest) -> AwpResponse {
        let key = route_key(&req.address);
        // Try exact match first (regional), then global fallback
        if let Some(handler) = self.routes.get(&key) {
            return handler(req);
        }
        // Global fallback: try without region
        if req.address.region.is_some() {
            let global_key = format!("{}.{}", req.address.name, req.address.category);
            if let Some(handler) = self.routes.get(&global_key) {
                return handler(req);
            }
        }
        AwpResponse::not_found()
    }

    /// Resolve a URI string to a response.
    pub fn resolve(&self, uri: &str) -> Result<AwpResponse, AwpError> {
        let addr = parse(uri)?;
        let req = AwpRequest {
            address: addr,
            method: crate::types::AwpMethod::Get,
            headers: vec![("AWP-Client".into(), "axon_awp/0.66.0".into())],
            body: vec![],
        };
        Ok(self.dispatch(&req))
    }

    /// Number of registered routes
    pub fn route_count(&self) -> usize { self.routes.len() }

    /// List all registered route keys
    pub fn list_routes(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.routes.keys().cloned().collect();
        keys.sort();
        keys
    }
}

impl Default for AwpRouter {
    fn default() -> Self { Self::new() }
}
