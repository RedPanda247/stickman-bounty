use bevy::prelude::*;

pub mod dash;
pub mod grapple;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((dash::DashPlugin, grapple::GrapplePlugin));
    }
}

// Re-exports for convenience
pub use dash::*;
pub use grapple::*;