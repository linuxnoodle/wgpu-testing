use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

use crate::{model, texture};
use crate::texture::generate_placeholder_texture;

/*#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}*/

pub async fn load_string(file_name: &str, subfolder: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(subfolder)
        .join(file_name);
    if !path.exists() {
        panic!("File at {:?} does not exist", path);
    }
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str, subfolder: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(subfolder)
        .join(file_name);
    if !path.exists() {
        panic!("File at {:?} does not exist", path);
    }
    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    subfolder: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name, subfolder).await?;
    if file_name.ends_with(".tga") {
        let img = image::load_from_memory_with_format(&data, image::ImageFormat::Tga).unwrap().flipv();
        texture::Texture::from_image(device, queue, &img, Some(file_name), is_normal_map)
    } else {
        texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
    }
}

pub async fn load_model_obj(
    file_name: &str,
    subfolder: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name, subfolder).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p, subfolder).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    ).await?;

    let mut materials = Vec::new();
    let mats = obj_materials?;
    for m in &mats {
        let diffuse_texture: texture::Texture;
        let normal_texture: texture::Texture; 

        if m.diffuse_texture.is_some() {
            println!("Loading texture: {}", m.diffuse_texture.as_ref().unwrap());
            diffuse_texture = load_texture(m.diffuse_texture.as_ref().unwrap(), subfolder, false, device, queue).await?;
        } else if let Some(diffuse) = m.diffuse { 
            println!("Loading BDSF diffuse: color ");
            diffuse_texture = generate_placeholder_texture(device,
                                                           queue,
                                                           128,
                                                           128,
                                                           false,
                                                           image::Rgba([
                                                                       (diffuse[0] * 255.0) as u8,
                                                                       (diffuse[1] * 255.0) as u8,
                                                                       (diffuse[2] * 255.0) as u8,
                                                                       255]
                                                           ));
        } else {
            println!("No diffuse texture found: defaulting to placeholder texture");
            diffuse_texture = generate_placeholder_texture(device,
                                                           queue,
                                                           128,
                                                           128,
                                                           true,
                                                           image::Rgba([
                                                                       0,
                                                                       0,
                                                                       0,
                                                                       255
                                                           ]));
        }
        
        if m.normal_texture.is_some(){
            let (_, norm) = m.normal_texture.as_ref().unwrap().rsplit_once(" ").unwrap();
            println!("Loading normal texture: {}", norm);
            normal_texture = load_texture(norm, subfolder, true, device, queue).await?;
        } else {
            println!("No normal texture found: defaulting to placeholder texture");
            // has to match diffuse_texture
            normal_texture = generate_placeholder_texture(device,
                                                          queue,
                                                          diffuse_texture.texture.width(),
                                                          diffuse_texture.texture.height(),
                                                          false, image::Rgba([
                                                                             0,
                                                                             0,
                                                                             0,
                                                                             255
                                                          ]));
        }

        materials.push(model::Material::new (
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ))
    }

    if mats.is_empty() {
        println!("No materials found! Falling back to placeholder material");
        let name = "Placeholder";
        materials.push(model::Material::new (
            device, 
            name,
            generate_placeholder_texture(
                device, 
                queue, 
                128,
                128,
                true,
                image::Rgba([
                             0,
                             0,
                             0,
                             255
                ]),
            ),
            generate_placeholder_texture(
                device, 
                queue, 
                128, 
                128,
                false,
                image::Rgba([
                             0,
                             0,
                             0,
                             255
                ]),
            ),
            layout, 
        ));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                vertices[c[0] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut vertices[i];
                v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

