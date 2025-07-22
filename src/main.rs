use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::keyboard::Keycode;
use sdl2::gfx::primitives::DrawRenderer;


const WORLD_UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };


fn main() {
    const RESOLUTION: [u32; 2] = [1200, 700];
    let sdl_context = sdl2::init().expect("Filed to initialize SDL context.");
    let video_subsystem = sdl_context.video().expect("Failed to initialize SDL video subsystem.");


    let mut window = video_subsystem.window("3d simulation", RESOLUTION[0], RESOLUTION[1])
            .position_centered()
            .build()
            .expect("Failed to build window.")
            .into_canvas()
            .build()
            .expect("Failed to convert window surface.");
    
    let mut events = sdl_context.event_pump().expect("Failed to build event pump.");

    let mut clock = clock::Clock::new(99999999);

    let mut running = true;

    let theta: f32 = 0.0;

    // OTHER STUFF
    let teapot_mesh = Mesh::from_str(include_str!("../assets/teapot.obj").to_string());

    let projection_matrix: Matrix4x4 = Matrix4x4::projection(RESOLUTION[1] as f32 / RESOLUTION[0] as f32, 1.0 / (90.0_f32  * 0.5).to_radians().tan(), 0.1, 1000.0);

    let mut camera: Camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 0.0, 0.0, 0.0);

    sdl_context.mouse().show_cursor(false);

    let mut mouse_locked = true;

    while running {
        let dt = clock.tick();

        for event in events.poll_iter() {
             match event {
                Event::Quit { .. } => { running = false; },
                Event::MouseMotion { xrel, yrel, .. } => { 
                    if mouse_locked {
                        camera.yaw -= xrel as f32 / 100.0; 
                        camera.pitch -= yrel as f32 / 100.0;
                    }
                },
                Event::KeyDown { keycode, .. } => {
                    match keycode.unwrap_or(Keycode::NUM_0) {
                        Keycode::ESCAPE => { 
                            mouse_locked = !mouse_locked; 
                            sdl_context.mouse().show_cursor(!mouse_locked);
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // updating
        if mouse_locked {
            sdl_context.mouse().warp_mouse_in_window(&window.window(), RESOLUTION[0] as i32 / 2, RESOLUTION[1] as i32 / 2);
        }

        let speed = 2.0;
        let keys_pressed = input::get_pressed_keys(&events);

        if keys_pressed.contains(&Keycode::UP) {
            camera.pos.y += speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::DOWN) {
            camera.pos.y -= speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::LEFT) {
            camera.pitch -= 1.0 * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::RIGHT) {
            camera.pitch += 1.0 * dt.as_secs_f32();
        }
        
        if keys_pressed.contains(&Keycode::W) {
            camera.pos += camera.get_front() * speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::S) {
            camera.pos -= camera.get_front() * speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::A) {
            camera.pos += camera.get_right() * speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::D) {
            camera.pos -= camera.get_right() * speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::SPACE) {
            camera.pos += WORLD_UP * speed * dt.as_secs_f32();
        }
        if keys_pressed.contains(&Keycode::LSHIFT) {
            camera.pos -= WORLD_UP * speed * dt.as_secs_f32();
        }


        camera.pitch = camera.pitch.clamp(-89.9_f32.to_radians(), 89.9_f32.to_radians());
        camera.yaw = camera.yaw % (2.0*std::f32::consts::PI);


        let rotation_z_matrix: Matrix4x4 = Matrix4x4::z_rotation(theta * 0.5);
        let rotation_x_matrix: Matrix4x4 = Matrix4x4::x_rotation(theta);
        let translation_matrix: Matrix4x4 = Matrix4x4::translation(0.0, -2.0, 4.0);
        let mut world_matrix: Matrix4x4 = rotation_z_matrix * rotation_x_matrix;
        world_matrix = world_matrix * translation_matrix;

        // update camera
        // let up: Vec3 = Vec3::new(0.0, 1.0, 0.0);
        // let camera_rotation_matrix = Matrix4x4::y_rotation(yaw) * Matrix4x4::x_rotation(pitch);
        // camera.look_direction = Vec3::from_vec4(Vec4::from_vec3(Vec3::new(0.0, 0.0, 1.0), 1.0) * camera_rotation_matrix);
        camera.look_at(camera.yaw, camera.pitch);
        let target: Vec3 = camera.pos + camera.look_direction;

        let view: Matrix4x4 = Matrix4x4::point_at_inverse(&Matrix4x4::point_at(camera.pos, target, camera.get_up()));

        // draw everything
        window.set_draw_color(Color::RGB(0, 0, 0));
        window.clear();

        let mut triangles_to_draw: Vec<Triangle> = Vec::new();

        for triangle in &teapot_mesh {
            let transformed_triangle = Triangle::new(
                Vec3::from_vec4(Vec4::from_vec3(triangle.points[0], 1.0) * world_matrix),
                Vec3::from_vec4(Vec4::from_vec3(triangle.points[1], 1.0) * world_matrix),
                Vec3::from_vec4(Vec4::from_vec3(triangle.points[2], 1.0) * world_matrix)
            );

            let line1 = transformed_triangle.points[1] - transformed_triangle.points[0];
            let line2 = transformed_triangle.points[2] - transformed_triangle.points[0];

            let mut normal: Vec3 = Vec3::new(
                line1.y * line2.z - line1.z * line2.y,
                line1.z * line2.x - line1.x * line2.z,
                line1.x * line2.y - line1.y * line2.x
            );
            normal.normalize();

            let camera_ray = transformed_triangle.points[0] - camera.pos;

            // Projection
            if normal.dot(&camera_ray) < 0.0 {
                // Lighting (very simple one)
                let mut light_direction: Vec3 = Vec3::new(0.0, 1.0, -1.0);
                light_direction.normalize();

                let dp = light_direction.dot(&normal).max(0.1);

                let color = Color::RGB((dp * 255.0) as u8, (dp * 255.0) as u8, (dp * 255.0) as u8);

                let mut viewed_triangle = Triangle::new(
                    Vec3::from_vec4(Vec4::from_vec3(transformed_triangle.points[0], 1.0) * view),
                    Vec3::from_vec4(Vec4::from_vec3(transformed_triangle.points[1], 1.0) * view),
                    Vec3::from_vec4(Vec4::from_vec3(transformed_triangle.points[2], 1.0) * view)
                );

                viewed_triangle.set_color(color);

                // clip the triangle against the near plane
                let clipped_triangles = viewed_triangle.clip_against_plane(Vec3::new(0.0, 0.0, 0.1), Vec3::new(0.0, 0.0, 1.0));


                for clipped_triangle in clipped_triangles {
                    // Actual projection
                    let projections: [Vec4; 3] = [
                        Vec4::from_vec3(clipped_triangle.points[0], 1.0) * projection_matrix,
                        Vec4::from_vec3(clipped_triangle.points[1], 1.0) * projection_matrix,
                        Vec4::from_vec3(clipped_triangle.points[2], 1.0) * projection_matrix
                    ];

                    let mut projected_triangle: Triangle = Triangle::new(
                        Vec3::from_vec4(projections[0]) / Vec3::new(projections[0].w, projections[0].w, projections[0].w),
                        Vec3::from_vec4(projections[1]) / Vec3::new(projections[1].w, projections[1].w, projections[1].w),
                        Vec3::from_vec4(projections[2]) / Vec3::new(projections[2].w, projections[2].w, projections[2].w)
                    );
                    projected_triangle.set_color(clipped_triangle.get_color());

                    projected_triangle.points[0] *= Vec3::new(-1.0, -1.0, 1.0);
                    projected_triangle.points[1] *= Vec3::new(-1.0, -1.0, 1.0);
                    projected_triangle.points[2] *= Vec3::new(-1.0, -1.0, 1.0);

                    // don't ask what is going on over here
                    let offset = Vec3::new(1.0, 1.0, 0.0);
                    projected_triangle.points[0] += offset;
                    projected_triangle.points[1] += offset;
                    projected_triangle.points[2] += offset;

                    let coef = Vec3::new(0.5 * RESOLUTION[0] as f32, 0.5 * RESOLUTION[1] as f32, 1.0);
                    projected_triangle.points[0] *= coef;
                    projected_triangle.points[1] *= coef;
                    projected_triangle.points[2] *= coef;

                    triangles_to_draw.push(projected_triangle);
                }
            }
        }

        triangles_to_draw.sort();
        triangles_to_draw.reverse();
        for triangle in triangles_to_draw {
            let mut triangle_list: Vec<Triangle> = Vec::new();
            triangle_list.push(triangle);
            let mut new_triangles = 1;

            for p in 0..4 {
                while new_triangles > 0 {
                    let test = triangle_list.remove(0);
                    new_triangles -= 1;

                    let mut new_triangles: Vec<Triangle> = match p {
                        0 => { test.clip_against_plane(Vec3::new(0.0,                        0.0,                        0.0), Vec3::new( 0.0,  1.0, 0.0)) },
                        1 => { test.clip_against_plane(Vec3::new(0.0,                        RESOLUTION[1] as f32 - 1.0, 0.0), Vec3::new( 0.0, -1.0, 0.0)) },
                        2 => { test.clip_against_plane(Vec3::new(0.0,                        0.0,                        0.0), Vec3::new( 1.0,  0.0, 0.0)) },
                        3 => { test.clip_against_plane(Vec3::new(RESOLUTION[0] as f32 - 1.0, 0.0,                        0.0), Vec3::new(-1.0,  0.0, 0.0)) },
                        _ => {panic!("HOW DID YOU MANAGE TO BREAK THE GAME")}
                    };

                    triangle_list.append(&mut new_triangles);
                }
                new_triangles = triangle_list.len();
            }
 
            for triangle in triangle_list {
                triangle.draw(&mut window);
            }
        }

        window.present();
    }
}



pub fn reverse_color(color: Color) -> Color {
    return Color::RGBA(color.a, color.b, color.g, color.r);
}


pub struct Camera {
    pub pos: Vec3,
    pub look_direction: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32
}
impl Camera {
    pub fn new(pos: Vec3, look_direction: Vec3, yaw: f32, pitch: f32, roll: f32) -> Self {
        return Self {
            pos,
            look_direction,
            yaw,
            pitch,
            roll
        }
    }

    pub fn look_at(&mut self, yaw: f32, pitch: f32) {
        self.look_direction.x = pitch.cos() * yaw.sin();
        self.look_direction.y = pitch.sin();
        self.look_direction.z = pitch.cos() * yaw.cos();
        self.look_direction.normalize();
    }

    pub fn get_up(&self) -> Vec3 {
        let forward = self.get_forward();
        let right = self.get_right();
        return forward.cross(&right);
    }

    pub fn get_right(&self) -> Vec3 {
        let forward = self.get_forward();
        return WORLD_UP.cross(&forward).normalized();
    }

    pub fn get_forward(&self) -> Vec3 {
        return self.look_direction;
    }

    // in contrast with `get_forward()`, this function shows where the "front" is. If you're a nerd, imagine it like a vector tied to the XZ plane
    pub fn get_front(&self) -> Vec3 {
        return Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
    }
}


#[derive(Copy, Clone)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}
impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        return Vec4 { x, y, z, w };
    }

    pub fn from_vec3(v: Vec3, w: f32) -> Self {
        return Self::new(v.x, v.y, v.z, w);
    }
}
impl std::ops::Mul<Matrix4x4> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Matrix4x4) -> Self::Output {
        return Vec4::new(
            self.x * rhs.mat[0][0] + self.y * rhs.mat[1][0] + self.z * rhs.mat[2][0] + rhs.mat[3][0], 
            self.x * rhs.mat[0][1] + self.y * rhs.mat[1][1] + self.z * rhs.mat[2][1] + rhs.mat[3][1], 
            self.x * rhs.mat[0][2] + self.y * rhs.mat[1][2] + self.z * rhs.mat[2][2] + rhs.mat[3][2],
            self.x * rhs.mat[0][3] + self.y * rhs.mat[1][3] + self.z * rhs.mat[2][3] + rhs.mat[3][3]
        );
    }
}

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self { x, y, z };
    }

    pub fn from_vec4(v: Vec4) -> Self {
        return Self::new(v.x, v.y, v.z);
    }

    pub fn normalize(&mut self) {
        let length = (self.x * self.x + self.y*self.y + self.z*self.z).sqrt();
        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    pub fn normalized(&self) -> Vec3 {
        let length = (self.x * self.x + self.y*self.y + self.z*self.z).sqrt();
        return Vec3::new(self.x / length, self.y / length, self.z / length);
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        return Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        );
    }

    pub fn plane_intersect(plane_p: Vec3, mut plane_n: Vec3, line_start: Vec3, line_end: Vec3) -> Self {
        plane_n.normalize();
        let plane_d = -(plane_n.dot(&plane_p));
        let ad = line_start.dot(&plane_n);
        let bd = line_end.dot(&plane_n);
        let t = (-plane_d - ad) / (bd - ad);
        let line_start_to_end = line_end - line_start;
        let line_to_intersect = line_start_to_end * t;
        return line_start + line_to_intersect;
    }
}
impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        return Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}
impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        return Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}
impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl std::ops::Mul for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Self) -> Self::Output {
        return Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z);
    }
}
impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self::Output {
        return Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs);
    }
}
impl std::ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl std::ops::Div for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Self) -> Self::Output {
        return Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z);
    }
}
impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.pad(format!("({}, {}, {})", self.x as i32, self.y as i32, self.z as i32).as_str());
    }
}




#[derive(Copy, Clone)]
pub struct Triangle {
    pub points: [Vec3; 3],
    color: Color
}
impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        return Self {
            points: [p1, p2, p3],
            color: Color::BLACK
        };
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        return self.color;
    }

    #[cfg(any(target_os = "macos"))]
    pub fn draw<T: RenderTarget>(&self, target: &mut Canvas<T>) {
        target.filled_trigon(self.points[0].x as i16, self.points[0].y as i16, self.points[1].x as i16, self.points[1].y as i16, self.points[2].x as i16, self.points[2].y as i16, reverse_color(self.get_color())).ok();
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    pub fn draw<T: RenderTarget>(&self, target: &mut Canvas<T>) {
        target.filled_trigon(self.points[0].x as i16, self.points[0].y as i16, self.points[1].x as i16, self.points[1].y as i16, self.points[2].x as i16, self.points[2].y as i16, self.get_color()).ok();
    }

    pub fn midpoint(&self) -> f32 {
        return (self.points[0].z + self.points[1].z + self.points[2].z) / 3.0;
    }

    pub fn clip_against_plane(&self, plane_p: Vec3, mut plane_n: Vec3) -> Vec<Triangle> {
        let mut results: Vec<Triangle> = Vec::new();

        plane_n.normalize();

        fn dist(plane_p: Vec3, plane_n: Vec3, p: Vec3) -> f32 {
            return plane_n.x * p.x + plane_n.y * p.y + plane_n.z * p.z - plane_n.dot(&plane_p);
        }

        let mut points_inside: Vec<Vec3> = Vec::new();
        let mut points_outside: Vec<Vec3> = Vec::new();
        let mut points_inside_count = 0;

        let d0 = dist(plane_p, plane_n, self.points[0]);
        let d1 = dist(plane_p, plane_n, self.points[1]);
        let d2 = dist(plane_p, plane_n, self.points[2]);


        if d0 >= 0.0 { points_inside_count += 1; points_inside.push(self.points[0]); }
        else { points_outside.push(self.points[0]); }

        if d1 >= 0.0 { points_inside_count += 1; points_inside.push(self.points[1]); }
        else { points_outside.push(self.points[1]); }

        if d2 >= 0.0 { points_inside_count += 1; points_inside.push(self.points[2]); }
        else { points_outside.push(self.points[2]); }

        if points_inside_count == 1 {
            let mut new_triangle = Triangle::new(
                points_inside[0],
                Vec3::plane_intersect(plane_p, plane_n, points_inside[0], points_outside[0]),
                Vec3::plane_intersect(plane_p, plane_n, points_inside[0], points_outside[1]),
            );

            new_triangle.set_color(self.get_color());
            results.push(new_triangle);
        }

        if points_inside_count == 2 {

            let mut new_triangle_1 = Triangle::new(
                points_inside[0],
                points_inside[1],
                Vec3::plane_intersect(plane_p, plane_n, points_inside[0], points_outside[0])
            );

            let mut new_triangle_2 = Triangle::new(
                points_inside[1],
                Vec3::plane_intersect(plane_p, plane_n, points_inside[1], points_outside[0]),
                Vec3::plane_intersect(plane_p, plane_n, points_inside[0], points_outside[0])
            );

            new_triangle_1.set_color(self.get_color());
            new_triangle_2.set_color(self.get_color());

            results.push(new_triangle_1);
            results.push(new_triangle_2);
        }

        if points_inside_count == 3 {
            results.push(*self);
        }

        return results;
    }
}
impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        return self.midpoint() == other.midpoint();
    }
}
impl Eq for Triangle {}
impl PartialOrd for Triangle {
    fn ge(&self, other: &Self) -> bool {
        return self.midpoint() >= other.midpoint();
    }

    fn gt(&self, other: &Self) -> bool {
        return self.midpoint() > other.midpoint();
    }

    fn le(&self, other: &Self) -> bool {
        return self.midpoint() <= other.midpoint();
    }

    fn lt(&self, other: &Self) -> bool {
        return self.midpoint() < other.midpoint();
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.midpoint() < other.midpoint() {
            return Some(std::cmp::Ordering::Less);
        }
        else if self.midpoint() > other.midpoint() {
            return Some(std::cmp::Ordering::Greater);
        }
        else {
            return Some(std::cmp::Ordering::Equal);
        }
    }
}
impl Ord for Triangle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.midpoint() < other.midpoint() {
            return std::cmp::Ordering::Less;
        }
        else if self.midpoint() > other.midpoint() {
            return std::cmp::Ordering::Greater;
        }
        else {
            return std::cmp::Ordering::Equal;
        }
    }
}


pub struct Mesh {
    triangles: Vec<Triangle>
}
impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        return Self { triangles };
    }

    pub fn from_str(str: String) -> Self {
        let mut triangles: Vec<Triangle> = Vec::new();

        let mut vertices: Vec<Vec3> = Vec::new();

        let data = str.split('\n');

        for line in data {
            let i: Vec<&str> = line.split(' ').into_iter().collect();
            if i[0] == "v" {
                let v: Vec3 = Vec3::new(
                    i[1].parse::<f32>().unwrap(), 
                    i[2].parse::<f32>().unwrap(), 
                    i[3].parse::<f32>().unwrap()
                );

                vertices.push(v);
            }
            else if i[0] == "f" {
                let triangle = Triangle::new(
                    vertices[i[1].parse::<usize>().unwrap() - 1], 
                    vertices[i[2].parse::<usize>().unwrap() - 1], 
                    vertices[i[3].parse::<usize>().unwrap() - 1]
                );

                triangles.push(triangle);
            }
        }

        return Self { triangles };
    }

    pub fn load_obj(filename: &str) -> Self {
        let content = std::fs::read_to_string(filename).expect(&format!("This file doesn't exist: {}", filename));

        return Self::from_str(content);
    }
}
impl<'a> IntoIterator for &'a Mesh {
    type Item = <std::slice::Iter<'a, Triangle> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, Triangle>;

    fn into_iter(self) -> Self::IntoIter {
        return self.triangles.as_slice().into_iter();
    }
}


#[derive(Copy, Clone)]
pub struct Matrix4x4 {
    pub mat: [[f32; 4]; 4]
}
impl Matrix4x4 {
    pub fn new(mat: [[f32; 4]; 4]) -> Self {
        return Self { mat };
    }

    pub fn empty() -> Self {
        return Self {
            mat: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0]
            ]
        }
    }

    pub fn identity() -> Self {
        return Self {
            mat: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]
        }
    }

    pub fn x_rotation(theta: f32) -> Self {
        return Self {mat: [
            [1.0,  0.0,                   0.0,                  0.0],
            [0.0,  (theta * 0.5).cos(),   (theta * 0.5).sin(),  0.0],
            [0.0,  -(theta * 0.5).sin(),  (theta * 0.5).cos(),  0.0],
            [0.0,  0.0,                   0.0,                  1.0]
        ]};
    }

    pub fn z_rotation(theta: f32) -> Self {
        return Self {mat: [
            [theta.cos(),   theta.sin(),  0.0,  0.0],
            [-theta.sin(),  theta.cos(),  0.0,  0.0],
            [0.0,           0.0,          1.0,  0.0],
            [0.0,           0.0,          0.0,  1.0]
        ]};
    }

    pub fn y_rotation(thetha: f32) -> Self {
        return Self {mat: [
            [thetha.cos(),   0.0,  thetha.sin(),  0.0],
            [0.0,            1.0,  0.0,           0.0],
            [-thetha.sin(),  0.0,  thetha.cos(),  0.0],
            [0.0,            0.0,  0.0,           1.0]
        ]};
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        return Self {mat: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x,   y,   z,   1.0]
        ]};
    }

    pub fn projection(aspect_ratio: f32, fov: f32, near: f32, far: f32) -> Self {
        return Self { mat: [
            [aspect_ratio * fov,  0.0,  0.0,                           0.0],
            [0.0,                 fov,  0.0,                           0.0],
            [0.0,                 0.0,  far / (far - near),            1.0],
            [0.0,                 0.0,  (-far * near) / (far - near),  0.0]
        ]};
    }

    pub fn point_at(pos: Vec3, target: Vec3, up: Vec3) -> Self {
        let mut new_forward = target - pos;
        new_forward.normalize();

        let a = new_forward * up.dot(&new_forward);
        let mut new_up = up - a;
        new_up.normalize();

        let new_right = new_up.cross(&new_forward);

        let matrix = Matrix4x4::new([
            [new_right.x,    new_right.y,    new_right.z,    0.0],
            [new_up.x,       new_up.y,       new_up.z,       0.0],
            [new_forward.x,  new_forward.y,  new_forward.z,  0.0],
            [pos.x,          pos.y,          pos.z,          1.0]
        ]);

        return matrix;
    }

    pub fn point_at_inverse(point_at: &Matrix4x4) -> Self {
        let m = point_at;
        return Matrix4x4::new([
            [m.mat[0][0], m.mat[1][0], m.mat[2][0], 0.0],
            [m.mat[0][1], m.mat[1][1], m.mat[2][1], 0.0],
            [m.mat[0][2], m.mat[1][2], m.mat[2][2], 0.0],
            [-(m.mat[3][0] * m.mat[0][0] + m.mat[3][1] * m.mat[0][1] + m.mat[3][2] * m.mat[0][2]), -(m.mat[3][0] * m.mat[1][0] + m.mat[3][1] * m.mat[1][1] + m.mat[3][2] * m.mat[1][2]), -(m.mat[3][0] * m.mat[2][0] + m.mat[3][1] * m.mat[2][1] + m.mat[3][2] * m.mat[2][2]), 1.0]
        ]);
    }
}
impl std::ops::Mul for Matrix4x4 {
    type Output = Matrix4x4;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = Self::empty();
        for c in 0..4 {
            for r in 0..4 {
                res.mat[r][c] = self.mat[r][0] * rhs.mat[0][c] + self.mat[r][1] * rhs.mat[1][c] + self.mat[r][2] * rhs.mat[2][c] + self.mat[r][3] * rhs.mat[3][c];
            }
        }
        return res;
    }
}

pub mod clock {
    use std::time::{Instant, Duration};
    use std::thread::sleep;

    pub struct Clock {
        frame_start: Instant,
        dt: Duration,
        pub fps: u32,
        frame_duration: Duration
    }
    impl Clock {
        pub fn new(fps: u32) -> Self {
            return Self {
                frame_start: Instant::now(),
                dt: Duration::new(0, 0),
                fps,
                frame_duration: Duration::new(0, 1_000_000_000_u32/fps)
            };
        }

        pub fn tick(&mut self) -> Duration {
            self.dt = self.frame_start.elapsed();
            self.frame_start = Instant::now();

            if self.dt < self.frame_duration {
                sleep(self.frame_duration - self.dt);
                return self.frame_duration;
            } else {
                return self.dt;
            }
        }
    }

    pub struct Timer {
        frame_start: Instant,
        dt: Duration,
        pub fps: u32,
        frame_duration: Duration
    }
    impl Timer {
        pub fn new(fps: u32) -> Self {
            return Self {
                frame_start: Instant::now(),
                dt: Duration::new(0, 0),
                fps,
                frame_duration: Duration::new(0, 1_000_000_000_u32/fps)
            }
        }

        pub fn tick(&mut self) -> bool {
            self.dt = self.frame_start.elapsed();

            if self.dt < self.frame_duration {
                return false;
            } else {
                self.frame_start = Instant::now().min(self.frame_start + self.frame_duration);
                return true;
            }
        }
    }

}

pub mod input {
    use std::collections::HashSet;
    use sdl2::keyboard::Keycode;
    use sdl2::EventPump;

    pub fn get_pressed_keys(events: &EventPump) -> HashSet<Keycode> {
        return events
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect();
    }
}
