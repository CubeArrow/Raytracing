use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::SplitWhitespace;
use std::sync::Arc;

use crate::hittables::hittable::Hittable;
use crate::hittables::triangle::Triangle;
use crate::material::Material;
use crate::textures::texture::Texture;
use crate::vec3::Vec3;

pub fn read_obj(path: &Path, material: Arc<Material>) -> Vec<Arc<Hittable>> {
    let mut file = std::fs::File::open(path).expect("Cannot open the file");
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut materials = HashMap::new();
    let mut texture_coordinates = Vec::new();
    let mut faces: Vec<Arc<Hittable>> = Vec::new();
    let mut current_mat = material;

    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        match words.next() {
            Some("mtllib") => {
                let mtllib_file = File::open(
                    path.parent().unwrap().to_str().unwrap().to_owned()
                        + "/"
                        + words.next().unwrap(),
                )
                    .unwrap();
                add_mtl(&mut materials, &mtllib_file)
            }
            Some("usemtl") => {
                current_mat = materials[words.next().unwrap()].clone();
            }
            Some("v") => {
                let vertex = parse_vec3(&mut words);
                vertices.push(vertex);
            }
            Some("vt") => {
                let mut vertex = (
                    parse_next_f64(words.borrow_mut()),
                    parse_next_f64(words.borrow_mut()),
                );
                texture_coordinates.push(vertex);
            }
            Some("f") => {
                let (a, texture_a) =
                    get_face_part(&mut vertices, &mut texture_coordinates, &mut words);
                let (b, texture_b) =
                    get_face_part(&mut vertices, &mut texture_coordinates, &mut words);
                let (c, texture_c) =
                    get_face_part(&mut vertices, &mut texture_coordinates, &mut words);
                let mut texture_coordinates = None;
                if texture_a != None {
                    let mut x = [(0.0, 0.0); 3];
                    x[0] = texture_a.unwrap();
                    x[1] = texture_b.unwrap();
                    x[2] = texture_c.unwrap();
                    texture_coordinates = Some(x);
                }
                faces.push(Arc::new(Hittable::Triangle {
                    triangle: Triangle::new_texture_coordinates(
                        a,
                        b,
                        c,
                        texture_coordinates,
                        current_mat.clone(),
                    ),
                }));
            }
            _ => {}
        }
    }
    return faces;
}

fn parse_vec3(mut words: &mut SplitWhitespace) -> Vec3 {
    let mut vertex = Vec3::new();
    for i in 0..3 {
        vertex.e[i] = parse_next_f64(words.borrow_mut());
    }
    vertex
}

pub fn add_mtl(map: &mut HashMap<String, Arc<Material>>, file: &File) { // TODO: Implement the correct Phong model
    let mut current_mat = Arc::new(Material::Diffuse { albedo: Texture::Solid { color: Vec3::new() }, emission: Vec3::new() });
    let mut current_name = String::from("");

    let mut diffuse = Vec3::new();

    let mut ambient = Vec3::new();
    let mut specular = Vec3::new();
    let mut specular_exp = 0.0;
    let mut dissolve = 0.0;
    let mut transmission_filter = Vec3::new();
    let mut refraction_index = 0.0;
    let mut emission = Vec3::new();

    let reader = BufReader::new(file);
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        match words.next() {
            None => {}
            Some("newmtl") => {
                add_material(map, &mut current_name, diffuse, specular, specular_exp, transmission_filter, refraction_index, emission);
                current_name =  words.next().unwrap().parse().unwrap();
            }
            Some("Kd") => {
                diffuse = parse_vec3(words.borrow_mut());
            }
            Some("Ks") => {
                specular = parse_vec3(words.borrow_mut());
            }
            Some("Ka") => {
                ambient = parse_vec3(words.borrow_mut());
            }
            Some("Ke") => {
                emission = parse_vec3(words.borrow_mut());
            }
            Some("Ni") => {
                refraction_index = parse_next_f64(words.borrow_mut());
            }
            Some("d") => {
                dissolve = parse_next_f64(words.borrow_mut());
            }
            Some("Tr") => {
                transmission_filter = parse_vec3(words.borrow_mut());
            }
            Some("Ns") => {
                specular_exp = parse_next_f64(words.borrow_mut());
            }
            _ => {}
        }
    }
    add_material(map, &mut current_name, diffuse, specular, specular_exp, transmission_filter, refraction_index, emission);
}

fn add_material(mut map: &mut HashMap<String, Arc<Material>>, mut current_name: &mut String, mut diffuse: Vec3, mut specular: Vec3, mut specular_exp: f64, mut transmission_filter: Vec3, mut refraction_index: f64, mut emission: Vec3) {
    let current_mat = if diffuse != Vec3::new() {
        Arc::new(Material::Diffuse { albedo: Texture::Solid { color: diffuse }, emission })
    } else if transmission_filter != Vec3::new() {
        Arc::new(Material::Dielectric { ir: refraction_index, tint: Texture::Solid { color: transmission_filter }, emission })
    } else {
        Arc::new(Material::Metal { albedo: Texture::Solid { color: specular }, fuzz: specular_exp / 1000.0, emission })
    };
    if current_name != "" {
        map.insert(current_name.parse().unwrap(), current_mat.clone());
    }
}

fn get_face_part(
    vertices: &mut Vec<Vec3>,
    texture_coordinates: &mut Vec<(f64, f64)>,
    words: &mut SplitWhitespace,
) -> (Vec3, Option<(f64, f64)>) {
    let x: Vec<&str> = words.next().unwrap().split("/").collect();
    let vertex;
    let mut texture_coordinate = None;
    match x.len() {
        1 => vertex = vertices[x[0].parse::<usize>().unwrap() - 1],
        _ => {
            vertex = vertices[x[0].parse::<usize>().unwrap() - 1];
            texture_coordinate = Some(texture_coordinates[x[0].parse::<usize>().unwrap() - 1]);
        }
    }
    return (vertex, texture_coordinate);
}

fn parse_next_f64(iteration: &mut SplitWhitespace) -> f64 {
    return iteration.next().unwrap().parse::<f64>().unwrap();
}
