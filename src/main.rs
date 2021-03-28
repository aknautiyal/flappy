use bracket_lib::prelude::*;
use std::{thread, time};
use rand::Rng;
const SCREEN_WIDTH : i32 = 80;
const SCREEN_HEIGHT : i32 = 50;
const FRAME_DURATION : f32 = 75.0;
const ENEMY_NO : i32 = 15;

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
   let a = x1 - x2;
   let b = y1 - y2;
   let d2:f32 = (a*a + b*b) as f32;

   d2.sqrt()
}

fn get_rnd_x()-> i32 {
    rand::thread_rng().gen_range(SCREEN_WIDTH, SCREEN_WIDTH+20)
}
fn get_rnd_y()-> i32 {
    rand::thread_rng().gen_range(1, SCREEN_HEIGHT - 1)
}

enum GameMode {
    Menu,
    Playing,
    End,
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

struct Enemy {
    x: i32,
    y: i32,
    active: bool,
}

impl Enemy {
    fn new(x: i32, y:i32) -> Self {
        Enemy {
            x,
            y,
            active: true,
        }
    }
    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(self.x, self.y, RED, BLACK, to_cp437('x'));
    }

    fn move_(&mut self) {
        self.x -= 1;
    }

    fn hit(&mut self, player: &Player) -> bool {
        if dist(player.x,player.y,self.x,self.y) < 1.5 {
            return true;
        }
        return false;
    }
}


impl Player {
    fn new(x: i32, y:i32) -> Self {
        Player {
            x,
            y,
            velocity:0.0,
        }
    }
    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(self.x, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        /*
        self.x += 1;
        if self.x > SCREEN_WIDTH {
            self.x = 0;
        }
        */
        if self.y < 0 {
            self.y = 0;
        }
    }
    fn flap(&mut self) {
        if self.velocity > 0.0 {
            self.velocity -= 2.0;
        }
    }

}

struct State {
    mode: GameMode,
    player: Player,
    enemy_vec: Vec<Enemy>,
    frame_time: f32,
    active_enemies: i32,
}

impl State {
    fn new() -> Self {
        State {
            frame_time: 0.0,
            mode: GameMode::Menu,
            player: Player::new(5,25),
            enemy_vec : Vec::with_capacity(ENEMY_NO as usize),
            active_enemies : 0,
        }
    }

    fn hit (&mut self) -> bool {
        for enemy in self.enemy_vec.iter_mut() {
            if enemy.active && enemy.hit(&self.player) {
                return true;
            }
        }

        return false;
    }
    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time  > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
            for enemy in self.enemy_vec.iter_mut() {
                if enemy.active {
                    enemy.move_();
                    if enemy.x <= 0 {
                        enemy.active = false;
                        self.active_enemies -= 1;
                    }
                }
            }
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        let mut x = 0;
        let y = SCREEN_HEIGHT -1;
        while x < SCREEN_WIDTH {
            let mut symbol1 = '|';
            let mut symbol2 = 'X';

            if self.frame_time as i32 % 2 == 0 {
                symbol1 = 'X';
                symbol2 = '|';
            }

            if x % 2 == 0 {
                ctx.set(x, y, GREEN, BLACK, to_cp437(symbol1));
            }
            else {
                ctx.set(x, y, GREEN, BLACK, to_cp437(symbol2));
            }
            x += 1;
        }
        self.player.render(ctx);
        if self.active_enemies == 0 {
            let active_enemies = rand::thread_rng().gen_range(ENEMY_NO / 2, ENEMY_NO);
            for n in 0..active_enemies {
                    self.enemy_vec.push(Enemy::new(get_rnd_x(), get_rnd_y()));
                    self.active_enemies += 1;
            }
        }

        for enemy in self.enemy_vec.iter_mut() {
            if enemy.active {
                enemy.render(ctx);
            }
        }
        ctx.print(0,0, "Press SPACE to flap.");
        ctx.print(SCREEN_WIDTH - 50 ,0, &format!("Altitude: {}", SCREEN_HEIGHT - self.player.y));
        if self.player.y > SCREEN_HEIGHT || self.hit() {
            let pause_sec = time::Duration::from_millis(2000);
            thread::sleep(pause_sec);
            self.mode = GameMode::End;
        }
    }
    fn restart(&mut self, _ctx:&mut BTerm) {
        self.player = Player::new(5,25);
        self.frame_time = 0.0;
        for enemy in self.enemy_vec.iter_mut() {
            enemy.active = false;
        }
        self.active_enemies = 0;
        self.mode = GameMode::Playing;

    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to the Flappy Game");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(ctx),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(8, " (P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(ctx),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self,  ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
