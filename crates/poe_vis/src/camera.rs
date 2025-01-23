use bevy::input::common_conditions::input_just_pressed;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

// Plugin definition
pub struct PoeVisCameraPlugin;

impl Plugin for PoeVisCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            // Configuration resource
            .init_resource::<CameraSettings>()
            .init_resource::<DragState>() // Add this line
            // Camera system dependencies
            .add_systems(Startup, (setup_camera, spawn_debug_text))
            .add_systems(
                Update,
                (camera_drag_system, camera_zoom_system, debug_camera_info),
            );
    }
}

// Add to your existing CameraSettings resource
#[derive(Resource)]
struct CameraSettings {
    pub drag_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            drag_sensitivity: 10.0,
            zoom_sensitivity: 0.15,
            min_zoom: 0.1,
            max_zoom: 1000.0,
        }
    }
}

// Camera drag state
#[derive(Resource, Default)]
struct DragState {
    active: bool,
    start_position: Vec2,
}

// Camera setup system
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 0.0, 100.0),
        GlobalTransform::default(),
    ));
}
// Zoom system implementation
fn camera_zoom_system(
    mut wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    settings: Res<CameraSettings>,
) {
    let mut projection = camera_query.single_mut();

    for event in wheel_events.read() {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 10.0, // Smoother zoom with line units
            MouseScrollUnit::Pixel => event.y,
        };

        // Apply zoom with sensitivity and clamping
        projection.scale = (projection.scale - delta * settings.zoom_sensitivity)
            .clamp(settings.min_zoom, settings.max_zoom);
    }
}
// Drag system
fn camera_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    windows: Query<&Window>,
) {
    let window = windows.single();

    if mouse_input.just_pressed(MouseButton::Left) {
        drag_state.active = true;
        if let Some(cursor_pos) = window.cursor_position() {
            drag_state.start_position = cursor_pos;
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        drag_state.active = false;
    }

    if drag_state.active {
        let mut total_delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            total_delta += event.delta;
        }

        if let Ok(mut transform) = camera_query.get_single_mut() {
            let sensitivity = 0.1; // Adjust sensitivity
            transform.translation.x -= total_delta.x * sensitivity;
            transform.translation.y += total_delta.y * sensitivity;
        }
    }
}

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont, TextLayout}; // Explicit text components

#[derive(Component)]
struct DebugTextMarker;

fn spawn_debug_text(mut commands: Commands) {
    commands.spawn((
        Text::new(""), // Empty initial text
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextLayout::default(),
        DebugTextMarker,
    ));
}

fn debug_camera_info(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
    mut text_query: Query<&mut Text, With<DebugTextMarker>>,
) {
    let window = windows.single();
    let (camera, camera_transform, projection) = camera_query.single();
    let mut text = text_query.single_mut();

    if let Some(cursor_pos) = window.cursor_position() {
        match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            Ok(world_pos) => {
                text.0 = format!(
                    "Cursor Screen: {:.1?}\nWorld: {:.1?}\nCamera: {:.1?}\nZoom: {:.2}",
                    cursor_pos,
                    world_pos,
                    camera_transform.translation().truncate(),
                    projection.scale
                );
            }
            Err(e) => {
                text.0 = format!("Projection Error: {:?}", e);
            }
        }
    }
}
