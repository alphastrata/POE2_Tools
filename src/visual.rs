
// src/visual.rs
use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text, TextFragment},
    input::keyboard::KeyCode,
    input::mouse::{MouseButton},
    Context, GameResult, event::EventHandler,
};
use crate::data::*;
use std::f64::consts::PI;

// Horizon palette
const BG_COLOR: Color   = Color { r: 0.07, g: 0.09, b: 0.12, a: 1.0 };
const EDGE_COLOR: Color = Color { r: 0.40, g: 0.40, b: 0.44, a: 1.0 };
const NODE_COLOR: Color = Color { r: 0.58, g: 0.76, b: 0.76, a: 1.0 };
const NOTABLE_COLOR:Color=Color { r: 1.00, g: 0.86, b: 0.42, a: 1.0 };
const ACTIVE_COLOR: Color=Color { r: 0.94, g: 0.40, b: 0.40, a: 1.0 };

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec2,  // keep as f32 internally
    pub zoom: f64,
}

pub struct TreeVisualization {
    pub data: TreeData,
    pub camera: Camera,
    pub hovered_node: Option<usize>,
    pub fuzzy_active: bool, // toggled by 'f' to open some fuzzy UI
    pub active_nodes: Vec<usize>,
    // bounding box
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl TreeVisualization {
    pub fn new(mut data: TreeData) -> Self {
        // fill (wx, wy) and other fields
        for (_, node) in data.passive_tree.nodes.iter_mut() {
            if let Some(group) = data.passive_tree.groups.get(&node.parent) {
                let r = node.radius as f64; // or your real orbit code
                let angle = (node.position as f64) * (2.0 * PI / 16.0);
                node.wx = group.x + r * angle.cos();
                node.wy = group.y + r * angle.sin();
            }
            if let Some(id) = &node.skill_id {
                if let Some(skill) = data.passive_skills.get(id) {
                    node.name = skill.name.clone().unwrap_or_default();
                    node.is_notable = skill.is_notable;
                    node.stats = skill.stats.clone();
                }
            }
            node.active = false; 
        }
        // bounding box
        let (mut min_x,mut max_x,mut min_y,mut max_y)=(999999.0,-999999.0,999999.0,-999999.0);
        for n in data.passive_tree.nodes.values() {
            if n.wx<min_x {min_x=n.wx;}
            if n.wx>max_x {max_x=n.wx;}
            if n.wy<min_y {min_y=n.wy;}
            if n.wy>max_y {max_y=n.wy;}
        }
        Self {
            data,
            camera: Camera { pos: Vec2::new(0.0, 0.0), zoom: 1.0 },
            hovered_node: None,
            fuzzy_active: false,
            active_nodes: vec![],
            min_x, max_x, min_y, max_y
        }
    }

    fn screen_to_world(&self, sx: f32, sy: f32, sw: f32, sh: f32) -> (f64, f64) {
        let cx = sw * 0.5;
        let cy = sh * 0.5;
        let wx = (sx - cx)/ (self.camera.zoom as f32) + self.camera.pos.x;
        let wy = (sy - cy)/ (self.camera.zoom as f32) + self.camera.pos.y;
        (wx as f64, wy as f64)
    }

    fn clamp_camera(&mut self, sw: f32, sh: f32) {
        // camera pos can't go past bounding box center
        let halfw = sw*0.5/self.camera.zoom as f32;
        let halfh = sh*0.5/self.camera.zoom as f32;
        // convert bounding box to f32
        let minx=self.min_x as f32; let maxx=self.max_x as f32;
        let miny=self.min_y as f32; let maxy=self.max_y as f32;
        if self.camera.pos.x < minx + halfw { self.camera.pos.x = minx + halfw; }
        if self.camera.pos.x > maxx - halfw { self.camera.pos.x = maxx - halfw; }
        if self.camera.pos.y < miny + halfh { self.camera.pos.y = miny + halfh; }
        if self.camera.pos.y > maxy - halfh { self.camera.pos.y = maxy - halfh; }
    }

    fn update_hover(&mut self, mx: f64, my: f64) {
        let mut best_dist = f64::MAX;
        let mut best_id = None;
        for (&id, node) in &self.data.passive_tree.nodes {
            let dx = node.wx - mx;
            let dy = node.wy - my;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist < 10.0 && dist < best_dist {
                best_dist = dist;
                best_id = Some(id);
            }
        }
        self.hovered_node = best_id;
    }
}

impl EventHandler for TreeVisualization {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, BG_COLOR);
        let (sw, sh) = ctx.gfx.drawable_size();
        
        // draw edges
        for node in self.data.passive_tree.nodes.values() {
            for cid in &node.connections {
                if let Some(other) = self.data.passive_tree.nodes.get(cid) {
                    let [sx1, sy1] = world_to_screen(
                        node.wx, node.wy,
                        self.camera.pos.x, self.camera.pos.y,
                        self.camera.zoom, sw, sh,
                    );
                    let [sx2, sy2] = world_to_screen(
                        other.wx, other.wy,
                        self.camera.pos.x, self.camera.pos.y,
                        self.camera.zoom, sw, sh,
                    );
                    let pts = [[sx1, sy1], [sx2, sy2]];
                    let line = Mesh::new_line(ctx, &pts, 2.0, EDGE_COLOR)?;
                    canvas.draw(&line, DrawParam::default());
                }
            }
        }
        
        // draw nodes
        let node_size = 0.006 * sh;
        for (id, node) in &self.data.passive_tree.nodes {
            let [sx, sy] = world_to_screen(
                node.wx, node.wy,
                self.camera.pos.x, self.camera.pos.y,
                self.camera.zoom, sw, sh,
            );
            let c = if node.active {
                ACTIVE_COLOR
            } else if node.is_notable {
                NOTABLE_COLOR
            } else {
                NODE_COLOR
            };
            let center = [sx, sy];
            let circle = Mesh::new_circle(ctx, DrawMode::fill(), center, node_size as f32, 0.2, c)?;
            canvas.draw(&circle, DrawParam::default());
        }
        
        // hover text
        if let Some(id) = self.hovered_node {
            if let Some(node) = self.data.passive_tree.nodes.get(&id) {
                let [sx, sy ]= world_to_screen(
                    node.wx, node.wy,
                    self.camera.pos.x, self.camera.pos.y,
                    self.camera.zoom, sw, sh,
                );
                let stats = node
                    .stats
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                let txt = Text::new(TextFragment::new(format!("{}\n{}", node.name, stats)).color(Color::WHITE));
                let dim = txt.measure(ctx)?;
                let r = Rect::new(
                    (sx + 8.0) as f32,
                    (sy - 4.0) as f32,
                    (dim.x + 8.0) as f32,
                    (dim.y + 8.0) as f32,
                );
                let bg = Mesh::new_rectangle(ctx, DrawMode::fill(), r, Color::from_rgba(0, 0, 0, 180))?;
                canvas.draw(&bg, DrawParam::default());
                canvas.draw(&txt, DrawParam::default().dest([sx + 12.0, sy]));
            }
        }
        
        canvas.finish(ctx)

    }

    fn key_down_event(&mut self, ctx: &mut Context, key: ggez::input::keyboard::KeyInput, _rpt: bool) -> GameResult {
        if let Some(kc)=key.keycode {
            let step=(50.0/self.camera.zoom) as f32;
            match kc {
                KeyCode::W=>self.camera.pos.y-=step,
                KeyCode::S=>self.camera.pos.y+=step,
                KeyCode::A=>self.camera.pos.x-=step,
                KeyCode::D=>self.camera.pos.x+=step,
                KeyCode::Escape=>ctx.request_quit(),
                KeyCode::F=>{
                    self.fuzzy_active=!self.fuzzy_active; 
                    // open fuzzy UI etc 
                }
                _=>{}
            }
        }
        let (sw,sh)=ctx.gfx.drawable_size();
        self.clamp_camera(sw,sh);
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, btn: MouseButton, x: f32, y: f32) -> GameResult {
        if btn==MouseButton::Left {
            let (sw,sh)=( _ctx.gfx.drawable_size() );
            let (wx,wy)=self.screen_to_world(x,y,sw,sh);
            // set active
            for (&id,node) in self.data.passive_tree.nodes.iter_mut() {
                let dx=node.wx-wx; let dy=node.wy-wy;
                if (dx*dx+dy*dy).sqrt()<10.0 {
                    node.active=true; 
                    self.active_nodes.push(id);
                }
            }
        }
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) -> GameResult {
        let (sw,sh)=ctx.gfx.drawable_size();
        let (wx,wy)=self.screen_to_world(x,y,sw,sh);
        self.update_hover(wx,wy);
        Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) -> GameResult {
        self.camera.zoom+=0.1*(y as f64);
        if self.camera.zoom<0.1 {self.camera.zoom=0.1;}
        if self.camera.zoom>100.0{self.camera.zoom=100.0;}
        let (sw,sh)=ctx.gfx.drawable_size();
        self.clamp_camera(sw,sh);
        Ok(())
    }
}

fn world_to_screen(wx: f64, wy: f64, camx: f32, camy: f32, zoom: f64, sw: f32, sh: f32)->[f32;2]{
    let cx=sw*0.5;
    let cy=sh*0.5;
    let sx=((wx as f32-camx)*(zoom as f32))+cx;
    let sy=((wy as f32-camy)*(zoom as f32))+cy;
    [sx,sy]
}
