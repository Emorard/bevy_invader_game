use bevy::{prelude::*, sprite::collide_aabb::{collide}};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(player_control_system.system())
        .add_system(missile_cooldown_system.system())
        .add_system(invader_movement_system.system())
        .add_system(missile_movement_system.system())
        .add_system(missile_collision_system.system())
        .run();
}

struct Myship {
    speed: f32,
    fired: bool,
    cooldown: Timer,
}

struct Invader;

enum Missile {
    Myship,
    Invader,
}

enum Collider {
    Invader,
    Myship,
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // spawn myship
        .spawn(SpriteBundle {
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_xyz(0.0, -215.0, 0.0),
            sprite: Sprite::new(Vec2::new(20.0, 20.0)),
            ..Default::default()
        })
        .with(Myship { 
            speed: 500.0 ,
            fired: true,
            cooldown: Timer::from_seconds(0.4, false),
        })
        .with(Collider::Myship);
        
        // spawn invaders
        let invader_rows = 5;
        let invader_columns = 12;
        let invader_spacing = 20.0;
        let invader_size = Vec2::new(25.0, 25.0);
        let invader_width = invader_columns as f32 * (invader_size.x + invader_spacing);
        
        // spawn location and material settings
        let invaders_offset = Vec3::new(-(invader_width - invader_size.x) / 2.0, 100.0, 0.0);
        let invader_material = materials.add(Color::BLUE.into());

        for row in 0..invader_rows {
            let y_position = row as f32 * (invader_size.y + invader_spacing);
            for column in 0..invader_columns {
                let invader_position = Vec3::new(
                    column as f32 * (invader_size.x + invader_spacing),
                    y_position,
                    0.0
                ) + invaders_offset;
                commands
                    // invader
                    .spawn(SpriteBundle {
                        material: invader_material.clone(),
                        sprite: Sprite::new(invader_size),
                        transform: Transform::from_translation(invader_position),
                        ..Default::default()
                    })
                    .with(Invader)
                    .with(Collider::Invader)
                    ;
            }
        }
}

fn player_control_system(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Myship, &mut Transform)>,
) {
    for (mut myship, mut transform) in query.iter_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }
        let translation = &mut transform.translation;

        translation.x += time.delta_seconds() * direction * myship.speed;
        translation.x = translation.x.min(300.0).max(-300.0);

        if keyboard_input.pressed(KeyCode::Space) {
            if !myship.fired {
                myship.fired = true;
                commands
                    .spawn(SpriteBundle {
                        material: materials.add(Color::WHITE.into()),
                        transform: Transform::from_xyz(translation.x, translation.y, 0.0),
                        sprite: Sprite::new(Vec2::new(3.0, 5.0)),
                        ..Default::default()
                    })
                    .with(Missile::Myship);
            }
        }
    }
}

fn missile_cooldown_system(
    time: Res<Time>,
    mut query: Query<&mut Myship>,
) {
    for mut myship in query.iter_mut() {
        myship.cooldown.tick(time.delta_seconds());
        if myship.cooldown.finished() && myship.fired {
            myship.fired = false;
            myship.cooldown.reset();
        }
    }
}

fn invader_movement_system(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut invader_query: Query<(&Invader, &mut Transform)>
) {
    // let delta_seconds = f32::min(0.2, time.delta_seconds());
    for (invader, mut transform) in invader_query.iter_mut() {
        // invader fire
        if rand::random::<i32>() % 10000 == 0 {
            commands
                .spawn(SpriteBundle {
                    material: materials.add(Color::WHITE.into()),
                    transform: Transform::from_xyz(transform.translation.x, transform.translation.y, 0.0),
                    sprite: Sprite::new(Vec2::new(3.0, 5.0)),
                    ..Default::default()
                })
                .with(Missile::Invader);
        }
    }
}

fn missile_movement_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut missile_query: Query<(Entity, &Missile, &mut Transform)>
) {
    let delta_seconds = f32::min(0.2, time.delta_seconds());
    for (entity, missile, mut transform) in missile_query.iter_mut() {
        match missile {
            Missile::Myship => {
                if  transform.translation.y > 400.0 {
                    commands.despawn(entity);
                }

                transform.translation.y += 400.0 * delta_seconds;
            }
            Missile::Invader => {
                if transform.translation.y < -400.0 {
                    commands.despawn(entity);
                }

                transform.translation.y -= 400.0 * delta_seconds;
            }
        }

    }
}

fn missile_collision_system(
    commands: &mut Commands,
    missile_query: Query<(Entity, &Missile, &Transform, &Sprite)>,
    collider_query: Query<(Entity, &Collider, &Transform, &Sprite)>
) {
    for (missile_entity, missile, missile_transform, sprite) in missile_query.iter() {
        let missile_size = sprite.size;
        // 
        for (collider_entity, collider, transform, sprite) in collider_query.iter() {
            
            match missile {
                Missile::Myship => {
                    if let Collider::Invader = collider {
                        let collision = collide(
                            missile_transform.translation,
                            missile_size,
                            transform.translation,
                            sprite.size
                        );
                        if let Some( _collision) = collision {
                            commands
                                .despawn(missile_entity)
                                .despawn(collider_entity);
                        }
                    }
                }
                Missile::Invader => {
                    if let Collider::Myship = collider {
                        if collide(
                            missile_transform.translation,
                            missile_size,
                            transform.translation,
                            sprite.size
                        ).is_some() {
                            commands
                                .despawn(missile_entity)
                                .despawn(collider_entity);
                        }
                    }
                }
            }
        }
    }
}
