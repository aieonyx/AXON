// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P66 — AWP protocol types: address, request, response, category registry

/// AWP category registry — fixed set, governed by community vote
/// Spec: awp://[name].[category] or awp://[name].[category].[region]
pub const CATEGORIES: &[&str] = &[
    "bank", "shop", "social", "news", "learn",
    "health", "dev", "art", "gov", "mesh", "id",
];

/// AWP address — sovereign alternative to domain name
#[derive(Debug, Clone, PartialEq)]
pub struct AwpAddress {
    /// Node name (alphanumeric + underscore, lowercase)
    pub name: String,
    /// Category from the fixed registry
    pub category: String,
    /// Optional ISO 3166-1 alpha-2 region code (e.g. "ph", "cz")
    pub region: Option<String>,
    /// Optional path after the address
    pub path: String,
}

impl AwpAddress {
    pub fn to_string(&self) -> String {
        let base = match &self.region {
            Some(r) => format!("awp://{}.{}.{}", self.name, self.category, r),
            None    => format!("awp://{}.{}", self.name, self.category),
        };
        if self.path.is_empty() || self.path == "/" {
            base
        } else {
            format!("{}{}", base, self.path)
        }
    }

    pub fn is_regional(&self) -> bool { self.region.is_some() }
    pub fn is_global(&self) -> bool { self.region.is_none() }
}

/// AWP request
#[derive(Debug, Clone)]
pub struct AwpRequest {
    pub address: AwpAddress,
    pub method: AwpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// AWP response
#[derive(Debug, Clone)]
pub struct AwpResponse {
    pub status: AwpStatus,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl AwpResponse {
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: AwpStatus::Ok,
            headers: vec![("Content-Type".into(), "text/axm".into())],
            body: body.into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            status: AwpStatus::NotFound,
            headers: vec![],
            body: b"awp: sovereign 404 - node not registered".to_vec(),
        }
    }
    pub fn forbidden() -> Self {
        Self {
            status: AwpStatus::Forbidden,
            headers: vec![],
            body: b"awp: access denied by sovereign policy".to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwpMethod {
    Get,
    Post,
    Sync,   // AWP-specific: mesh data synchronization
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwpStatus {
    Ok          = 200,
    NotFound    = 404,
    Forbidden   = 403,
    BadRequest  = 400,
    MeshError   = 503,
}

/// AWP error types
#[derive(Debug, Clone, PartialEq)]
pub enum AwpError {
    InvalidScheme(String),
    InvalidCategory(String),
    InvalidRegion(String),
    InvalidName(String),
    MalformedAddress(String),
    RouteNotFound(String),
}

impl std::fmt::Display for AwpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidScheme(s)    => write!(f, "invalid scheme: {}", s),
            Self::InvalidCategory(s)  => write!(f, "invalid category: {}", s),
            Self::InvalidRegion(s)    => write!(f, "invalid region: {}", s),
            Self::InvalidName(s)      => write!(f, "invalid name: {}", s),
            Self::MalformedAddress(s) => write!(f, "malformed AWP address: {}", s),
            Self::RouteNotFound(s)    => write!(f, "route not found: {}", s),
        }
    }
}
