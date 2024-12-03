use std::f32::consts::PI;

use ::rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use macroquad::prelude::*;

use crate::{HEIGHT, WIDTH};

const ENTITY_DETECT_RANGE: f32 = 100.0;
const ENTITY_SIZE: f32 = 5.0;
const FOOD_SIZE: f32 = 3.0;

trait QuadTreeItem: Clone {
    fn pos(&self) -> Vec2;
    fn draw(&self);
    fn debug(&self);
}

#[derive(Debug, Clone)]
pub struct Entity {
    pos: Vec2,
    coop: f32,
    share: f32,
    direction: f32,
    food_collected: u32,
    group: EntityType,
}

#[derive(Debug, Clone)]
enum EntityType {
    Predator,
    Prey,
}

impl Distribution<EntityType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EntityType {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..=1) {
            // rand 0.8
            0 => EntityType::Predator,
            1 => EntityType::Prey,
            _ => EntityType::Prey,
        }
    }
}

impl QuadTreeItem for Entity {
    fn pos(&self) -> Vec2 {
        self.pos
    }
    fn draw(&self) {
        draw_circle(
            self.pos.x,
            self.pos.y,
            ENTITY_SIZE,
            match &self.group {
                EntityType::Predator => RED,
                EntityType::Prey => BLUE,
            },
        );

        draw_line(
            self.pos.x,
            self.pos.y,
            self.pos.x + ENTITY_SIZE * 1.5 * self.direction.cos(),
            self.pos.y + ENTITY_SIZE * 1.5 * self.direction.sin(),
            2.0,
            WHITE,
        );
    }

    fn debug(&self) {
        // draw_circle_lines(self.pos.x, self.pos.y, ENTITY_DETECT_RANGE, 2.0, WHITE);

        // draw_rectangle_lines(
        //     self.pos.x - ENTITY_DETECT_RANGE,
        //     self.pos.y - ENTITY_DETECT_RANGE,
        //     2.0 * ENTITY_DETECT_RANGE,
        //     2.0 * ENTITY_DETECT_RANGE,
        //     2.0,
        //     WHITE,
        // );
    }
}

impl Entity {
    pub fn new(x: f32, y: f32, group: EntityType) -> Self {
        Entity {
            pos: Vec2 { x, y },
            coop: rand::gen_range(0.0, 1.0),
            share: rand::gen_range(0.0, 1.0),
            group,
            direction: rand::gen_range(-PI, PI),
            food_collected: 0,
        }
    }

    fn step(&mut self, entities_qt: &QuadTree<Entity>, foods_qt: &QuadTree<Food>) {
        let Vec2 { x, y } = self.pos;

        let close_entities: Vec<Entity> = entities_qt
            .query(Rect {
                x: x - ENTITY_DETECT_RANGE,
                y: y - ENTITY_DETECT_RANGE,
                w: 2.0 * ENTITY_DETECT_RANGE,
                h: 2.0 * ENTITY_DETECT_RANGE,
            })
            .into_iter()
            .filter(|e| {
                self.pos.distance_squared(e.pos) <= ENTITY_DETECT_RANGE * ENTITY_DETECT_RANGE
            })
            .collect();

        match &self.group {
            EntityType::Prey => {
                let mut close_foods: Vec<Food> = foods_qt
                    .query(Rect::new(
                        x - ENTITY_DETECT_RANGE,
                        y - ENTITY_DETECT_RANGE,
                        2.0 * ENTITY_DETECT_RANGE,
                        2.0 * ENTITY_DETECT_RANGE,
                    ))
                    .into_iter()
                    .filter(|e| {
                        self.pos.distance_squared(e.pos)
                            <= ENTITY_DETECT_RANGE * ENTITY_DETECT_RANGE
                    })
                    .collect();

                close_foods.sort_by(|a, b| {
                    self.pos
                        .distance(a.pos)
                        .partial_cmp(&self.pos.distance(b.pos))
                        .unwrap()
                });

                // println!("{:?}", &close_foods);

                if close_foods.len() > 0 {
                    for food in close_foods[0..=(2.min(close_foods.len() - 1))].iter() {
                        let Vec2 {
                            x: food_x,
                            y: food_y,
                        } = food.pos();

                        draw_line(
                            self.pos.x,
                            self.pos.y,
                            food_x,
                            food_y,
                            2.0,
                            Color::new(
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0,
                            ),
                        );

                        let dir =
                            (food.pos.y - self.pos.y).atan2(close_foods[0].pos.x - self.pos.x);
                        self.direction += steer(self.direction, dir)
                            / self.pos.distance(food.pos)
                            / (if self.food_collected != 0 {
                                self.food_collected as f32
                            } else {
                                1.0
                            })
                            .powf(1.5);
                    }
                }
            }
            EntityType::Predator => {
                let mut close_preies: Vec<Entity> = close_entities
                    .into_iter()
                    .filter(|e| {
                        (self.pos.distance_squared(e.pos)
                            <= ENTITY_DETECT_RANGE * ENTITY_DETECT_RANGE)
                            && match e.group {
                                EntityType::Prey => true,
                                EntityType::Predator => false,
                            }
                    })
                    .collect();

                close_preies.sort_by(|a, b| {
                    self.pos
                        .distance(a.pos)
                        .partial_cmp(&self.pos.distance(b.pos))
                        .unwrap()
                });

                if close_preies.len() > 0 {
                    for food in close_preies[0..=(2.min(close_preies.len() - 1))].iter() {
                        let Vec2 {
                            x: food_x,
                            y: food_y,
                        } = food.pos();

                        draw_line(
                            self.pos.x,
                            self.pos.y,
                            food_x,
                            food_y,
                            2.0,
                            Color::new(
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0 / (if self.food_collected != 0 {
                                    self.food_collected as f32
                                } else {
                                    1.0
                                }),
                                1.0,
                            ),
                        );

                        let dir =
                            (food.pos.y - self.pos.y).atan2(close_preies[0].pos.x - self.pos.x);
                        self.direction += steer(self.direction, dir)
                            / self.pos.distance(food.pos)
                            / (if self.food_collected != 0 {
                                self.food_collected as f32
                            } else {
                                1.0
                            })
                            .powf(1.5);
                    }
                }
            }
        }

        self.pos.x += 1.0 * self.direction.cos();
        self.pos.y += 1.0 * self.direction.sin();

        self.pos.x = self.pos.x.rem_euclid(entities_qt.boundary.w);
        self.pos.y = self.pos.y.rem_euclid(entities_qt.boundary.h);
    }
}

#[derive(Clone, Debug)]
pub struct Food {
    pos: Vec2,
    is_eaten: bool,
}

impl QuadTreeItem for Food {
    fn pos(&self) -> Vec2 {
        self.pos
    }
    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, FOOD_SIZE, ORANGE);
    }
    fn debug(&self) {}
}

impl Food {
    pub fn new(x: f32, y: f32) -> Self {
        Food {
            pos: Vec2 { x, y },
            is_eaten: false,
        }
    }

    fn step(&mut self, entity_qt: &QuadTree<Entity>) {
        let Vec2 { x, y } = self.pos;

        let mut close_entities: Vec<Entity> = entity_qt
            .query(Rect::new(
                x - ENTITY_DETECT_RANGE,
                y - ENTITY_DETECT_RANGE,
                2.0 * ENTITY_DETECT_RANGE,
                2.0 * ENTITY_DETECT_RANGE,
            ))
            .into_iter()
            .filter(|e| {
                self.pos.distance_squared(e.pos) <= ENTITY_DETECT_RANGE * ENTITY_DETECT_RANGE
            })
            .collect();

        close_entities.sort_by(|a, b| {
            self.pos
                .distance(a.pos)
                .partial_cmp(&self.pos.distance(b.pos))
                .unwrap()
        });

        if close_entities.len() > 0 {
            if self.pos.distance_squared(close_entities[0].pos) <= (ENTITY_SIZE + FOOD_SIZE).powi(2)
            {
                self.is_eaten = true;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct QuadTreeChildren<T: QuadTreeItem> {
    /* (x,y)----+----+
           + nw + ne +
           +----+----+
           + sw + se +
           +----+----(x+w,y+h)
    */
    nw: QuadTree<T>,
    ne: QuadTree<T>,
    sw: QuadTree<T>,
    se: QuadTree<T>,
}

impl<T: QuadTreeItem> QuadTreeChildren<T> {
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        let Rect { x, y, w, h } = boundary;

        QuadTreeChildren {
            nw: QuadTree::new(
                Rect {
                    x,
                    y,
                    w: w / 2.0,
                    h: h / 2.0,
                },
                capacity,
            ),
            ne: QuadTree::new(
                Rect {
                    x: x + w / 2.0,
                    y,
                    w: w / 2.0,
                    h: h / 2.0,
                },
                capacity,
            ),
            sw: QuadTree::new(
                Rect {
                    x,
                    y: y + h / 2.0,
                    w: w / 2.0,
                    h: h / 2.0,
                },
                capacity,
            ),
            se: QuadTree::new(
                Rect {
                    x: x + w / 2.0,
                    y: y + boundary.h / 2.0,
                    w: w / 2.0,
                    h: h / 2.0,
                },
                capacity,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuadTree<T: QuadTreeItem> {
    boundary: Rect,
    capacity: usize,
    items: Vec<T>,
    children: Option<Box<QuadTreeChildren<T>>>,
    divided: bool,
}

impl<T: QuadTreeItem> QuadTree<T> {
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        QuadTree {
            boundary,
            capacity,
            items: vec![],
            children: None,
            divided: false,
        }
    }

    pub fn insert(&mut self, item: T) {
        if self.boundary.contains(item.pos()) {
            if (self.capacity >= self.items.len()) && (!self.divided) {
                self.items.push(item);
            } else {
                if !self.divided {
                    self.subdivide();
                }

                if let Some(children) = self.children.as_mut() {
                    children.ne.insert(item.clone());
                    children.nw.insert(item.clone());
                    children.se.insert(item.clone());
                    children.sw.insert(item.clone());
                }
            }
        }
    }

    fn subdivide(&mut self) {
        self.children = Some(Box::new(QuadTreeChildren::new(
            self.boundary,
            self.capacity,
        )));
        for entity in self.items.iter() {
            if let Some(children) = self.children.as_mut() {
                children.ne.insert(entity.clone());
                children.nw.insert(entity.clone());
                children.se.insert(entity.clone());
                children.sw.insert(entity.clone());
            }
        }
        self.items.clear();
        self.divided = true;
    }

    pub fn query(&self, range: Rect) -> Vec<T> {
        let mut found: Vec<T> = Vec::new();
        if self.boundary.intersect(range).is_some() {
            if let Some(children) = self.children.as_ref() {
                let mut ne_result = children.ne.query(range);
                let mut nw_result = children.nw.query(range);
                let mut se_result = children.se.query(range);
                let mut sw_result = children.sw.query(range);

                found.append(&mut ne_result);
                found.append(&mut nw_result);
                found.append(&mut se_result);
                found.append(&mut sw_result);
            } else {
                for entity in self.items.iter() {
                    if range.contains(entity.pos()) {
                        found.push(entity.clone());
                    }
                }
            }
        }
        found
    }

    fn show(&self) {
        let Rect { x, y, w, h } = self.boundary;
        draw_rectangle_lines(x, y, w, h, 2.0, BLUE);
        if self.query(self.boundary).len() != 0 {
            draw_text(
                format!("{}", self.query(self.boundary).len()).as_str(),
                x + w - 30.0,
                y + 20.0,
                16.0,
                BLUE,
            );
        }
        if let Some(children) = self.children.as_ref() {
            children.ne.show();
            children.nw.show();
            children.se.show();
            children.sw.show();
        }
        for item in self.items.iter() {
            item.debug();
        }
    }

    fn draw(&self) {
        for entity in self.items.iter() {
            entity.draw();
        }
        if let Some(children) = self.children.as_ref() {
            children.ne.draw();
            children.nw.draw();
            children.se.draw();
            children.sw.draw();
        }
    }
}

pub struct Simulation {
    pub is_running: bool,
    pub entity_qt: QuadTree<Entity>,
    pub entities: Vec<Entity>,
    pub food_qt: QuadTree<Food>,
    pub foods: Vec<Food>,
    boundary: Rect,
    capacity: usize,
    debug: bool,
    pause: bool,
}

impl Simulation {
    pub fn new(w: f32, h: f32) -> Self {
        Simulation {
            is_running: false,
            entities: vec![],
            entity_qt: QuadTree::new(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w,
                    h,
                },
                4,
            ),
            foods: vec![],
            food_qt: QuadTree::new(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w,
                    h,
                },
                4,
            ),
            boundary: Rect {
                x: 0.0,
                y: 0.0,
                w,
                h,
            },
            capacity: 4,
            debug: false,
            pause: false,
        }
    }

    fn clear(&self) {
        clear_background(BLACK);
    }

    fn draw(&self) {
        // for entity in self.entities.iter() {
        //     entity.draw(GREEN);
        // }

        // if root_ui().button(Vec2::new(0.0, 0.0), "asdasd") {
        //     println!("{:?}", self.quadtree);
        // }
        let range = Rect {
            x: 600.0,
            y: 600.0,
            w: 130.0,
            h: 98.0,
        };
        let Rect { x, y, w, h } = range;
        let result = self.entity_qt.query(range).len();
        draw_rectangle_lines(x, y, w, h, 2.0, RED);
        draw_text(
            format!("{}, {}, {:?}", self.entities.len(), result, self.pause).as_str(),
            30.0,
            30.0,
            16.0,
            WHITE,
        );
    }

    fn update(&mut self) {
        if is_mouse_button_pressed(MouseButton::Left)
            || (is_mouse_button_down(MouseButton::Left) && is_key_down(KeyCode::LeftControl))
        {
            let (x, y) = mouse_position();
            let rand_len = ((rand::gen_range(0.0, 1.0) as f64).sqrt() as f32) * 20.0;
            let rand_dir = rand::gen_range(-PI, PI);
            self.entities.push(Entity::new(
                x + rand_len * rand_dir.cos(),
                y + rand_len * rand_dir.sin(),
                EntityType::Prey,
            ));
        }
        if is_mouse_button_pressed(MouseButton::Right)
            || (is_mouse_button_down(MouseButton::Right) && is_key_down(KeyCode::LeftControl))
        {
            let (x, y) = mouse_position();
            let rand_len = ((rand::gen_range(0.0, 1.0) as f64).sqrt() as f32) * 20.0;
            let rand_dir = rand::gen_range(-PI, PI);
            self.entities.push(Entity::new(
                x + rand_len * rand_dir.cos(),
                y + rand_len * rand_dir.sin(),
                EntityType::Predator,
            ));
        }
        if is_mouse_button_down(MouseButton::Middle) {
            let (x, y) = mouse_position();
            let rand_len = ((rand::gen_range(0.0, 1.0) as f64).sqrt() as f32) * 20.0;
            let rand_dir = rand::gen_range(-PI, PI);
            self.foods.push(Food::new(
                x + rand_len * rand_dir.cos(),
                y + rand_len * rand_dir.sin(),
            ));
        }

        if is_key_pressed(KeyCode::R) {
            self.entities.clear();
            self.foods.clear();
            for _ in 0..100 {
                self.entities.push(Entity::new(
                    rand::gen_range(0.0, WIDTH as f32),
                    rand::gen_range(0.0, HEIGHT as f32),
                    EntityType::Prey,
                ));
            }
            for _ in 0..100 {
                self.foods.push(Food::new(
                    rand::gen_range(0.0, WIDTH as f32),
                    rand::gen_range(0.0, HEIGHT as f32),
                ));
            }
        }
        if is_key_pressed(KeyCode::D) {
            self.debug = !self.debug;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.is_running = false;
        }
        if is_key_pressed(KeyCode::Space) {
            self.pause = !self.pause;
        }

        self.entity_qt = QuadTree::new(self.boundary.clone(), self.capacity);
        for entity in self.entities.iter_mut() {
            if !self.pause {
                entity.step(&self.entity_qt, &self.food_qt);
            }
            self.entity_qt.insert(entity.clone());
        }
        self.food_qt = QuadTree::new(self.boundary.clone(), self.capacity);
        for food in self.foods.iter_mut() {
            if !self.pause {
                food.step(&self.entity_qt);
            }
            if !food.is_eaten {
                self.food_qt.insert(food.clone());
            }
            // food.step(&self.food_qt);
        }
    }

    pub fn frame(&mut self) {
        self.clear();
        self.update();
        self.draw();
        self.entity_qt.draw();
        self.food_qt.draw();
        if self.debug {
            self.entity_qt.show();
            self.food_qt.show();
        }
    }
}

fn steer(dir0: f32, dir: f32) -> f32 {
    if (dir - dir0).rem_euclid(2.0 * PI) < PI {
        1.0
    } else {
        -1.0
    }
}
