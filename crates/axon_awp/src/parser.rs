// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P66 — AWP URI parser: awp://name.category[.region][/path]

use crate::types::{AwpAddress, AwpError, CATEGORIES};

/// ISO 3166-1 alpha-2 region codes (subset — common sovereign regions)
/// Full set can be loaded from EdisonDB in production.
pub const REGIONS: &[&str] = &[
    "af","ax","al","dz","as","ad","ao","ai","aq","ag","ar","am","aw","au","at",
    "az","bs","bh","bd","bb","by","be","bz","bj","bm","bt","bo","bq","ba","bw",
    "bv","br","io","bn","bg","bf","bi","cv","kh","cm","ca","ky","cf","td","cl",
    "cn","cx","cc","co","km","cg","cd","ck","cr","ci","hr","cu","cw","cy","cz",
    "dk","dj","dm","do","ec","eg","sv","gq","er","ee","sz","et","fk","fo","fj",
    "fi","fr","gf","pf","tf","ga","gm","ge","de","gh","gi","gr","gl","gd","gp",
    "gu","gt","gg","gn","gw","gy","ht","hm","va","hn","hk","hu","is","in","id",
    "ir","iq","ie","im","il","it","jm","jp","je","jo","kz","ke","ki","kp","kr",
    "kw","kg","la","lv","lb","ls","lr","ly","li","lt","lu","mo","mg","mw","my",
    "mv","ml","mt","mh","mq","mr","mu","yt","mx","fm","md","mc","mn","me","ms",
    "ma","mz","mm","na","nr","np","nl","nc","nz","ni","ne","ng","nu","nf","mk",
    "mp","no","om","pk","pw","ps","pa","pg","py","pe","ph","pn","pl","pt","pr",
    "qa","re","ro","ru","rw","bl","sh","kn","lc","mf","pm","vc","ws","sm","st",
    "sa","sn","rs","sc","sl","sg","sx","sk","si","sb","so","za","gs","ss","es",
    "lk","sd","sr","sj","se","ch","sy","tw","tj","tz","th","tl","tg","tk","to",
    "tt","tn","tr","tm","tc","tv","ug","ua","ae","gb","us","um","uy","uz","vu",
    "ve","vn","vg","vi","wf","eh","ye","zm","zw",
];

/// Parse an AWP URI into an AwpAddress.
/// Formats:
///   awp://name.category
///   awp://name.category/path
///   awp://name.category.region
///   awp://name.category.region/path
pub fn parse(uri: &str) -> Result<AwpAddress, AwpError> {
    // Strip scheme
    let rest = uri.strip_prefix("awp://")
        .ok_or_else(|| AwpError::InvalidScheme(uri.to_string()))?;

    // Split path
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].to_string()),
        None    => (rest, "/".to_string()),
    };

    // Split authority by '.'
    let parts: Vec<&str> = authority.splitn(3, '.').collect();
    if parts.len() < 2 {
        return Err(AwpError::MalformedAddress(uri.to_string()));
    }

    let name = parts[0].to_lowercase();
    let category = parts[1].to_lowercase();

    // Validate name
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AwpError::InvalidName(name));
    }

    // Validate category
    if !CATEGORIES.contains(&category.as_str()) {
        return Err(AwpError::InvalidCategory(category));
    }

    // Optional region
    let region = if parts.len() == 3 {
        let r = parts[2].to_lowercase();
        if r.len() != 2 || !r.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(AwpError::InvalidRegion(r));
        }
        if !REGIONS.contains(&r.as_str()) {
            return Err(AwpError::InvalidRegion(r));
        }
        Some(r)
    } else {
        None
    };

    Ok(AwpAddress { name, category, region, path })
}

/// Check if a string is a valid AWP URI (no allocation on false).
pub fn is_awp(uri: &str) -> bool {
    uri.starts_with("awp://")
}
