use bevy::{gltf::Gltf, prelude::*};
use bevy_rapier3d::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource)]
pub struct Scene(Handle<Gltf>);

// Startup: Load assets/Scene.glb as a resource
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Scene(asset_server.load("Scene.glb")))
}

// Update/Loading: Fetch the first scene out of the asset
fn spawn_level(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    // Used to queue up a state change
    mut next_state: ResMut<NextState<AppState>>,
    // A handle to the loaded scene
    gltf_handle: Res<Scene>,
    // All the GLTF assets in the app resources
    gltf: Res<Assets<Gltf>>,
) {
    match app_state.get() {
        AppState::Loading => {
            // Wait for the scene to be loaded into the ECS
            if let Some(gltf) = gltf.get(&gltf_handle.0) {
                // Drop the loaded GLTF into the ECS without transforming it
                commands.spawn(SceneBundle {
                    scene: gltf.default_scene.clone().unwrap(),
                    ..Default::default()
                });
                // We only want to run this once, so advance the app state
                next_state.set(AppState::Running);
            }
        }
        _ => {}
    }
}

// Update/Running: Add physics to the scene meshes
fn mesh_hook(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    // This is only run when entities are Added
    added_name: Query<(Entity, &Name, &Children), Added<Name>>,
    // All the mesh assets in the app reosurces
    meshes: Res<Assets<Mesh>>,
    // Handles to all the meshes
    mesh_handles: Query<&Handle<Mesh>>,
) {
    match app_state.get() {
        AppState::Running => {
            for (entity, name, children) in &added_name {
                // GLTF nodes don't have a tagging system, so we have to use their names
                if name.as_str() == "Plane" {
                    // Our plane only has 1 mesh, but just in case...
                    for collider in children
                        .iter()
                        // Filter out everything that isn't a mesh (materials, etc.)
                        .filter_map(|entity| {
                            mesh_handles
                                .get(*entity)
                                .ok()
                        })
                        // Convert the mesh into a rapier collider, silently ignoring bad meshes
                        .filter_map(|mesh_handle| {
                            Collider::from_bevy_mesh(
                                meshes.get(mesh_handle).unwrap(),
                                &ComputedColliderShape::TriMesh,
                            )
                        })
                    {
                        // Add the collider to the ECS as a child of the Plane
                        commands.entity(entity).insert(collider);
                    }
                }
                // Matches CubeA, CubeB, and CubeC
                if name.contains("Cube") {
                    for collider in children
                        .iter()
                        .filter_map(|entity| mesh_handles.get(*entity).ok())
                        .filter_map(|mesh_handle| {
                            Collider::from_bevy_mesh(
                                meshes.get(mesh_handle).unwrap(),
                                &ComputedColliderShape::TriMesh,
                            )
                        })
                    {
                        // Add a rapier RigidBody to the ECS
                        let rigid_body = commands
                            .spawn(RigidBody::Dynamic)
                        // Without this component, the rapier physics won't be applied to child entities
                        // Bevy warns us about this: https://bevyengine.org/learn/errors/#b0004
                            .insert(SpatialBundle::default())
                            .id();
                        // Reparent the Cube to the RigidBody, and add the collider to it
                        commands
                            .entity(entity)
                            .set_parent(rigid_body)
                            .insert(collider);
                    }
                }
            }
        }
        _ => {}
    }
}

fn main() {
    App::new()
        // Everything needed for rendering, windowing, etc.
        .add_plugins(DefaultPlugins)
        // The Rapier 3D physics simulation
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // Gives us a view into the Rapier colliders as wireframes
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_level, mesh_hook))
        .run()
}
