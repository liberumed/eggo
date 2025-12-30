use bevy::{app::AppExit, prelude::*};

use crate::components::*;
use crate::resources::{GameState, Stats};

pub fn update_counters(
    stats: Res<Stats>,
    mut philosophy_query: Query<&mut Text, (With<PhilosophyCounter>, Without<NatureStudyCounter>, Without<WisdomCounter>)>,
    mut nature_query: Query<&mut Text, (With<NatureStudyCounter>, Without<PhilosophyCounter>, Without<WisdomCounter>)>,
    mut wisdom_query: Query<&mut Text, (With<WisdomCounter>, Without<PhilosophyCounter>, Without<NatureStudyCounter>)>,
) {
    if let Ok(mut text) = philosophy_query.single_mut() {
        **text = stats.philosophy.to_string();
    }
    if let Ok(mut text) = nature_query.single_mut() {
        **text = stats.nature_study.to_string();
    }
    if let Ok(mut text) = wisdom_query.single_mut() {
        **text = stats.wisdom.to_string();
    }
}

pub fn update_hp_text(
    health_query: Query<(&Health, &Children, Option<&Dead>), Or<(With<Player>, With<Creature>)>>,
    mut text_query: Query<(&mut Text2d, &mut Visibility), With<HpText>>,
    mut heart_query: Query<&mut Visibility, (With<HeartSprite>, Without<HpText>)>,
) {
    for (health, children, dead) in &health_query {
        for child in children.iter() {
            if let Ok((mut text, mut visibility)) = text_query.get_mut(child) {
                if dead.is_some() {
                    *visibility = Visibility::Hidden;
                } else {
                    **text = health.0.to_string();
                }
            }
            if let Ok(mut visibility) = heart_query.get_mut(child) {
                if dead.is_some() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn stabilize_text_rotation(
    parent_query: Query<(&Transform, &Children), Or<(With<Player>, With<Creature>)>>,
    mut text_query: Query<&mut Transform, (With<HpText>, Without<Player>, Without<Creature>)>,
) {
    for (parent_transform, children) in &parent_query {
        let inverse_rotation = parent_transform.rotation.inverse();
        for child in children.iter() {
            if let Ok(mut text_transform) = text_query.get_mut(child) {
                text_transform.rotation = inverse_rotation;
            }
        }
    }
}

pub fn show_death_screen(
    player_query: Query<&Dead, With<Player>>,
    mut death_screen_query: Query<&mut Visibility, With<DeathScreen>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if player_query.iter().next().is_some() {
        if let Ok(mut visibility) = death_screen_query.single_mut() {
            *visibility = Visibility::Inherited;
        }
        next_state.set(GameState::Dead);
    }
}

pub fn handle_new_game_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    mut exit: MessageWriter<AppExit>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).spawn();
            }
            exit.write(AppExit::Success);
        }
    }
}
