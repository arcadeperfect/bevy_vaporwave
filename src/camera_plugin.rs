use bevy::prelude::*;

pub struct cam_plugin;

impl Plugin for cam_plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, rotate_camera);
    }
}

fn setup(mut commands: Commands) {

    let h = 1.5;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, h, 5.0).looking_at(Vec3::new(0., h, 0.), Vec3::Y),
            ..default()
        },
        OrbitCamera {
            radius: 5.0,
            speed: 1.0,
        },
        RotationEnabled(false),  // Start with rotation enabled
    ));
}


#[derive(Component)]
struct RotationEnabled(bool);



#[derive(Component)]
struct OrbitCamera {
    radius: f32,
    speed: f32,
}

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &OrbitCamera, &mut RotationEnabled), With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Toggle rotation when 'R' is pressed
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (_, _, mut rotation_enabled) in query.iter_mut() {
            rotation_enabled.0 = !rotation_enabled.0;
        }
    }

    for (mut transform, orbit, rotation_enabled) in query.iter_mut() {
        if rotation_enabled.0 {
            let angle = time.elapsed_seconds() * orbit.speed;
            let x = orbit.radius * angle.cos();
            let z = orbit.radius * angle.sin();

            transform.translation = Vec3::new(x, 0.0, z);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}


// fn rotate_camera(
//     time: Res<Time>,
//     mut query: Query<(&mut Transform, &OrbitCamera), With<Camera>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
// ) {



//     for (mut transform, orbit) in query.iter_mut() {
//         let angle = time.elapsed_seconds() * orbit.speed;
//         let x = orbit.radius * angle.cos();
//         let z = orbit.radius * angle.sin();

//         transform.translation = Vec3::new(x, 0.0, z);
//         transform.look_at(Vec3::ZERO, Vec3::Y);
//     }
// }
