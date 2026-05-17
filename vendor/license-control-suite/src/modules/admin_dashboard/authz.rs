pub fn can_read(scopes: &[String]) -> bool {
    scopes.iter().any(|s| s == "admin:read" || s == "admin:reset:write")
}

pub fn can_write_reset(scopes: &[String]) -> bool {
    scopes.iter().any(|s| s == "admin:reset:write")
}
