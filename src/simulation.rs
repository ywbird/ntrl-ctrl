use std::f32::consts::PI;

use ::rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use macroquad::{
    input,
    prelude::*,
    ui::{hash, root_ui, widgets},
};

use ultraviolet;

use crate::{HEIGHT, WIDTH};

#[derive(Debug, Clone)]
pub struct Entity {
    pos: Vec2,
    coop: f32,
    share: f32,
    direction: f32,
    is_eaten: bool,
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

impl Entity {
    pub fn new(x: f32, y: f32) -> Self {
        Entity {
            pos: Vec2 { x, y },
            coop: rand::gen_range(0.0, 1.0),
            share: rand::gen_range(0.0, 1.0),
            group: ::rand::random(),
            direction: rand::gen_range(-PI, PI),
            is_eaten: false,
        }
    }

    fn draw(&self) {
        draw_circle(
            self.pos.x,
            self.pos.y,
            5.0,
            match &self.group {
                EntityType::Predator => RED,
                EntityType::Prey => BLUE,
            },
        );
        draw_line(
            self.pos.x,
            self.pos.y,
            self.pos.x + 5.0 * self.direction.cos(),
            self.pos.y + 5.0 * self.direction.sin(),
            4.0,
            BLACK,
        );
    }

    fn step(&mut self, world: &QuadTree) {
        match &self.group {
            EntityType::Prey => {}
            EntityType::Predator => {}
        }

        self.pos.x += 1.0 * self.direction.cos();
        self.pos.y += 1.0 * self.direction.sin();

        self.pos.x = self.pos.x.rem_euclid(world.boundary.w);
        self.pos.y = self.pos.y.rem_euclid(world.boundary.h);
    }
}

#[derive(Debug, Clone)]
struct QuadTreeChildren {
    /* (x,y)----+----+
           + nw + ne +
           +----+----+
           + sw + se +
           +----+----(x+w,y+h)
    */
    nw: QuadTree,
    ne: QuadTree,
    sw: QuadTree,
    se: QuadTree,
}

impl QuadTreeChildren {
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
pub struct QuadTree {
    boundary: Rect,
    capacity: usize,
    entities: Vec<Entity>,
    children: Option<Box<QuadTreeChildren>>,
    divided: bool,
}

impl QuadTree {
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        QuadTree {
            boundary,
            capacity,
            entities: vec![],
            children: None,
            divided: false,
        }
    }

    pub fn insert(&mut self, entity: Entity) {
        if self.boundary.contains(entity.pos) {
            if (self.capacity >= self.entities.len()) && (!self.divided) {
                self.entities.push(entity);
            } else {
                if !self.divided {
                    self.subdivide();
                }

                if let Some(children) = self.children.as_mut() {
                    children.ne.insert(entity.clone());
                    children.nw.insert(entity.clone());
                    children.se.insert(entity.clone());
                    children.sw.insert(entity.clone());
                }
            }
        }
    }

    fn subdivide(&mut self) {
        self.children = Some(Box::new(QuadTreeChildren::new(
            self.boundary,
            self.capacity,
        )));
        for entity in self.entities.iter() {
            if let Some(children) = self.children.as_mut() {
                children.ne.insert(entity.clone());
                children.nw.insert(entity.clone());
                children.se.insert(entity.clone());
                children.sw.insert(entity.clone());
            }
        }
        self.entities.clear();
        self.divided = true;
    }

    // TODO: 실제 엔티티 수와 맞지 않는 문제 발생
    pub fn query(&self, range: Rect) -> Vec<&Entity> {
        let mut found: Vec<&Entity> = Vec::new();
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
                for entity in self.entities.iter() {
                    if range.contains(entity.pos) {
                        found.push(entity);
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
    }

    fn draw(&self) {
        for entity in self.entities.iter() {
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
    pub quadtree: QuadTree,
    pub entities: Vec<Entity>,
    boundary: Rect,
    capacity: usize,
    debug: bool,
    pause: bool,
}

impl Simulation {
    pub fn new(w: f32, h: f32) -> Self {
        Simulation {
            entities: vec![],
            is_running: false,
            quadtree: QuadTree::new(
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
        let result = self.quadtree.query(range).len();
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
        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();
            let rand_len = rand::gen_range(-30.0, 30.0);
            let rand_dir = rand::gen_range(-PI, PI);
            self.entities.push(Entity::new(
                x + rand_len * rand_dir.cos(),
                y + rand_len * rand_dir.sin(),
            ));
        }
        if is_mouse_button_down(MouseButton::Right) {
            let (x, y) = mouse_position();
            self.entities.push(Entity::new(x, y));
        }

        if is_key_down(KeyCode::R) {
            self.entities.clear();
            for _ in 0..100 {
                self.entities.push(Entity::new(
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

        if self.pause {
            return ();
        }
        self.quadtree = QuadTree::new(self.boundary.clone(), self.capacity);
        for entity in self.entities.iter_mut() {
            self.quadtree.insert(entity.clone());
            entity.step(&self.quadtree);
        }
    }

    pub fn frame(&mut self) {
        self.clear();
        self.update();
        self.draw();
        self.quadtree.draw();
        if self.debug {
            self.quadtree.show();
        }
    }
}
