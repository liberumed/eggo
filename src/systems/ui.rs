use bevy::prelude::*;

use crate::components::*;
use crate::resources::{GameState, Stats};
use crate::spawners::{spawn_creatures, spawn_ground_item, spawn_player, spawn_target_outline, CharacterAssets};

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
    mut shadow_query: Query<&mut Transform, (With<Shadow>, Without<Player>, Without<Creature>)>,
) {
    for (parent_transform, children) in &parent_query {
        let inverse_rotation = parent_transform.rotation.inverse();
        let inverse_scale = Vec3::new(
            1.0 / parent_transform.scale.x,
            1.0 / parent_transform.scale.y,
            1.0,
        );
        for child in children.iter() {
            if let Ok(mut shadow_transform) = shadow_query.get_mut(child) {
                shadow_transform.rotation = inverse_rotation;
                shadow_transform.scale = inverse_scale;
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

pub fn handle_new_game_button(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    entities_query: Query<Entity, Or<(With<Player>, With<Creature>, With<BloodParticle>, With<TargetOutline>, With<GroundItem>)>>,
    mut death_screen_query: Query<&mut Visibility, With<DeathScreen>>,
    mut stats: ResMut<Stats>,
    mut next_state: ResMut<NextState<GameState>>,
    assets: Res<CharacterAssets>,
    registry: Res<ItemRegistry>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            for entity in &entities_query {
                commands.entity(entity).despawn();
            }

            stats.philosophy = 0;
            stats.nature_study = 0;
            stats.wisdom = 0;

            spawn_player(&mut commands, &assets, &mut meshes, &mut materials);
            spawn_target_outline(&mut commands, &assets);
            spawn_creatures(&mut commands, &assets, &mut meshes, &mut materials);

            // Spawn test items
            spawn_ground_item(&mut commands, &assets, &registry, ItemId::HealthPotion, 1, Vec2::new(30.0, 20.0));
            spawn_ground_item(&mut commands, &assets, &registry, ItemId::HealthPotion, 3, Vec2::new(-40.0, 30.0));
            spawn_ground_item(&mut commands, &assets, &registry, ItemId::RustyKnife, 1, Vec2::new(50.0, -10.0));

            if let Ok(mut visibility) = death_screen_query.single_mut() {
                *visibility = Visibility::Hidden;
            }

            next_state.set(GameState::Playing);
        }
    }
}
