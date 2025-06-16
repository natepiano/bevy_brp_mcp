// This module is reserved for formatting utilities
// Currently empty after migrating to JSON responses

use crate::constants::BRP_REGISTRATION_REQUIREMENTS;

/// Generates a hint message for when no components are found.
///
/// # Arguments
/// * `entity_id` - Optional entity ID for entity-specific queries
///
/// # Returns
/// A formatted hint message explaining why components might not be visible
pub fn generate_empty_components_hint(entity_id: Option<u64>) -> String {
    if let Some(id) = entity_id {
        format!(
            "No components found for entity {}. Possible reasons:\n\
            1. The entity doesn't exist\n\
            2. The entity has no components registered for BRP access\n\
            3. Components aren't properly configured for reflection\n\n\
            {}",
            id, BRP_REGISTRATION_REQUIREMENTS
        )
    } else {
        format!(
            "No components found. This usually means no components are properly configured for BRP.\n\n\
            {}",
            BRP_REGISTRATION_REQUIREMENTS
        )
    }
}
