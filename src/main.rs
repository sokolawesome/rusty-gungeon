use bevy::prelude::*;

mod player;
use player::PlayerPlugin;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Gungeon".into(),
                resolution: (1280.0, 720.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PlayerPlugin)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu_stub)
        .add_systems(OnEnter(GameState::InGame), setup_ingame_stub)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_main_menu_stub() {
    info!("entered mainmenu state (stub)");
}

fn setup_ingame_stub() {
    info!("entered ingame state (stub)");
}
