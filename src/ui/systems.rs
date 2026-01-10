use bevy::prelude::*;

use super::components::*;
use crate::inventory::weapons::{PlayerWeapon, Weapon};
use crate::core::{Dead, GameAction, GameState, Health, InputBindings, Shadow};
use crate::creatures::Creature;
use crate::player::{Player, Stats};

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

pub fn stabilize_shadow(
    parent_query: Query<(&Transform, &Children), Or<(With<Player>, With<Creature>)>>,
    mut shadow_query: Query<(&Shadow, &mut Transform), (Without<Player>, Without<Creature>)>,
) {
    for (parent_transform, children) in &parent_query {
        let inverse_rotation = parent_transform.rotation.inverse();
        for child in children.iter() {
            if let Ok((shadow, mut shadow_transform)) = shadow_query.get_mut(child) {
                shadow_transform.rotation = inverse_rotation;
                // Apply inverse scale multiplied by base_scale
                shadow_transform.scale = Vec3::new(
                    shadow.base_scale.x / parent_transform.scale.x,
                    shadow.base_scale.y / parent_transform.scale.y,
                    1.0,
                );
            }
        }
    }
}

// Toggle pause menu on Esc
pub fn toggle_pause_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if bindings.just_pressed(GameAction::Pause, &keyboard, &mouse) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

// Show menu when entering Paused state
pub fn show_pause_menu(
    mut menu_query: Query<&mut Visibility, With<GameMenu>>,
    mut title_query: Query<&mut Text, With<MenuTitle>>,
    mut title_color_query: Query<&mut TextColor, With<MenuTitle>>,
    mut resume_query: Query<&mut Visibility, (With<ResumeButton>, Without<GameMenu>)>,
) {
    if let Ok(mut visibility) = menu_query.single_mut() {
        *visibility = Visibility::Inherited;
    }
    if let Ok(mut text) = title_query.single_mut() {
        **text = "PAUSED".to_string();
    }
    if let Ok(mut color) = title_color_query.single_mut() {
        *color = TextColor(Color::srgb(0.9, 0.9, 0.9));
    }
    if let Ok(mut visibility) = resume_query.single_mut() {
        *visibility = Visibility::Inherited;
    }
}

// Hide menu when exiting Paused state
pub fn hide_pause_menu(
    mut menu_query: Query<&mut Visibility, With<GameMenu>>,
) {
    if let Ok(mut visibility) = menu_query.single_mut() {
        *visibility = Visibility::Hidden;
    }
}

// Show death menu when player dies
pub fn show_death_menu(
    player_query: Query<&Dead, With<Player>>,
    mut menu_query: Query<&mut Visibility, With<GameMenu>>,
    mut title_query: Query<&mut Text, With<MenuTitle>>,
    mut title_color_query: Query<&mut TextColor, With<MenuTitle>>,
    mut resume_query: Query<&mut Visibility, (With<ResumeButton>, Without<GameMenu>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if player_query.iter().next().is_some() {
        if let Ok(mut visibility) = menu_query.single_mut() {
            *visibility = Visibility::Inherited;
        }
        if let Ok(mut text) = title_query.single_mut() {
            **text = "YOU DIED".to_string();
        }
        if let Ok(mut color) = title_color_query.single_mut() {
            *color = TextColor(Color::srgb(0.9, 0.2, 0.2));
        }
        if let Ok(mut visibility) = resume_query.single_mut() {
            *visibility = Visibility::Hidden;
        }
        next_state.set(GameState::Dead);
    }
}

pub fn update_weapon_info(
    weapon_query: Query<&Weapon, With<PlayerWeapon>>,
    mut name_query: Query<&mut Text, (With<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponSpeedText>, Without<WeaponRangeText>, Without<WeaponConeText>, Without<WeaponKnockbackText>, Without<WeaponTypeText>)>,
    mut damage_query: Query<&mut Text, (With<WeaponDamageText>, Without<WeaponNameText>, Without<WeaponSpeedText>, Without<WeaponRangeText>, Without<WeaponConeText>, Without<WeaponKnockbackText>, Without<WeaponTypeText>)>,
    mut speed_query: Query<&mut Text, (With<WeaponSpeedText>, Without<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponRangeText>, Without<WeaponConeText>, Without<WeaponKnockbackText>, Without<WeaponTypeText>)>,
    mut range_query: Query<&mut Text, (With<WeaponRangeText>, Without<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponSpeedText>, Without<WeaponConeText>, Without<WeaponKnockbackText>, Without<WeaponTypeText>)>,
    mut cone_query: Query<&mut Text, (With<WeaponConeText>, Without<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponSpeedText>, Without<WeaponRangeText>, Without<WeaponKnockbackText>, Without<WeaponTypeText>)>,
    mut knockback_query: Query<&mut Text, (With<WeaponKnockbackText>, Without<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponSpeedText>, Without<WeaponRangeText>, Without<WeaponConeText>, Without<WeaponTypeText>)>,
    mut type_query: Query<&mut Text, (With<WeaponTypeText>, Without<WeaponNameText>, Without<WeaponDamageText>, Without<WeaponSpeedText>, Without<WeaponRangeText>, Without<WeaponConeText>, Without<WeaponKnockbackText>)>,
) {
    let Ok(weapon) = weapon_query.single() else { return };

    if let Ok(mut text) = name_query.single_mut() {
        **text = weapon.name.clone();
    }
    if let Ok(mut text) = damage_query.single_mut() {
        **text = weapon.damage.to_string();
    }
    if let Ok(mut text) = speed_query.single_mut() {
        **text = weapon.speed.to_string();
    }
    if let Ok(mut text) = range_query.single_mut() {
        **text = weapon.reach.to_string();
    }
    if let Ok(mut text) = cone_query.single_mut() {
        **text = weapon.arc.to_string();
    }
    if let Ok(mut text) = knockback_query.single_mut() {
        **text = format!("{:.0}", weapon.knockback_force());
    }
    if let Ok(mut text) = type_query.single_mut() {
        **text = format!("{:?}", weapon.attack_type);
    }
}

// Resume button handler
pub fn handle_resume_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

// New Game button handler
pub fn handle_menu_new_game_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<MenuNewGameButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut new_game_requested: ResMut<crate::core::NewGameRequested>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            match current_state.get() {
                GameState::Paused => {
                    // Go to Dead first, flag will trigger auto-transition to Playing
                    new_game_requested.0 = true;
                    next_state.set(GameState::Dead);
                }
                GameState::Dead => {
                    next_state.set(GameState::Playing);
                }
                _ => {}
            }
        }
    }
}

// Auto-transition from Dead to Playing when new game was requested from pause
pub fn auto_start_new_game(
    mut next_state: ResMut<NextState<GameState>>,
    mut new_game_requested: ResMut<crate::core::NewGameRequested>,
) {
    if new_game_requested.0 {
        new_game_requested.0 = false;
        next_state.set(GameState::Playing);
    }
}

// Exit button handler
pub fn handle_exit_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ExitButton>)>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            app_exit.write(AppExit::Success);
        }
    }
}

/// Spawn key bindings guide panel (bottom-left)
pub fn spawn_key_bindings_panel(mut commands: Commands) {
    commands
        .spawn((
            KeyBindingsPanel,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.7)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|parent| {
            let bindings = [
                ("WASD", "Move"),
                ("Shift", "Sprint"),
                ("Space", "Dash"),
                ("LMB", "Attack"),
                ("RMB", "Block"),
                ("R", "Weapon"),
                ("Tab", "Inventory"),
                ("Esc", "Pause"),
            ];
            for (key, action) in bindings {
                parent
                    .spawn(Node {
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(key),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.75, 0.5)),
                        ));
                        row.spawn((
                            Text::new(action),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    });
            }
        });
}
