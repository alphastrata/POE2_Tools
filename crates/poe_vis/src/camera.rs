use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::text::{TextFont, TextLayout};

use crate::events::MoveCameraReq;
use crate::resources::{CameraSettings, DragState};

// Plugin definition
pub struct PoeVisCameraPlugin;

impl Plugin for PoeVisCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .init_resource::<DragState>()
            .add_systems(Startup, (setup_camera, spawn_debug_text))
            .add_systems(
                Update,
                (
                    (camera_drag_system, camera_zoom_system).run_if(ui_unlocked),
                    // debug_camera_info,
                    move_camera_to_target_system.run_if(on_event::<MoveCameraReq>),
                ),
            );
        log::debug!("PoeVisCamera plugin enabled");
    }
}

fn ui_unlocked(settings: Res<CameraSettings>) -> bool {
    !settings.egui_has_lock
}

fn move_camera_to_target_system(
    mut move_requests: EventReader<MoveCameraReq>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    mut ortho_q: Query<&mut OrthographicProjection, With<Camera2d>>,
    settings: Res<CameraSettings>,
) {
    move_requests.read().for_each(|MoveCameraReq(target)| {
        let mut transform = camera_q.single_mut();
        let mut ortho = ortho_q.single_mut();
        transform.translation.x = target.x;
        transform.translation.y = target.y;
        transform.translation.z = 0.0;
        ortho.scale = target.z.clamp(settings.min_zoom, settings.max_zoom);
    });
}

// Updated camera setup system
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scale: 35.0,
            near: -1000.0,
            far: 1000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: ScalingMode::WindowSize,
            area: Rect::from_center_size(Vec2::ZERO, Vec2::new(1.0, 1.0)),
        },
        Msaa::Sample8,
    ));
}

// Zoom system implementation
fn camera_zoom_system(
    mut wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    settings: Res<CameraSettings>,
) {
    let mut projection = camera_query.single_mut();

    wheel_events.read().for_each(|event| {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 10.0, // Smoother zoom with line units
            MouseScrollUnit::Pixel => event.y,
        };

        // Apply zoom with sensitivity and clamping
        projection.scale = (projection.scale - delta * settings.zoom_sensitivity)
            .clamp(settings.min_zoom, settings.max_zoom);
    });
}

fn camera_drag_system(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    windows: Query<&Window>,
    settings: Res<CameraSettings>, // Add this parameter
) {
    let window = windows.single();

    if mouse_input.just_pressed(MouseButton::Middle) {
        drag_state.active = true;
        if let Some(cursor_pos) = window.cursor_position() {
            drag_state.start_position = cursor_pos;
        }
    }

    if mouse_input.just_released(MouseButton::Middle) {
        drag_state.active = false;
    }

    if drag_state.active {
        let mut total_delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            total_delta += event.delta;
        }

        if let Ok(mut transform) = camera_query.get_single_mut() {
            // Use the setting from CameraSettings
            transform.translation.x -= total_delta.x * settings.drag_sensitivity;
            transform.translation.y += total_delta.y * settings.drag_sensitivity;
        }
    }
}

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
                    "Cursor Screen: {:.1?}\nWorld: {:.1?}\nCamera: {:.1?}\nZoom: {:.2}\nZ: {:.2}",
                    cursor_pos,
                    world_pos,
                    camera_transform.translation(), // Full Vec3 including Z
                    projection.scale,
                    camera_transform.translation().z // Explicit Z position
                );
            }
            Err(e) => {
                text.0 = format!("Projection Error: {e:?}");
            }
        }
    }
}
