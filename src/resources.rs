use std::{io::{BufReader, Cursor}, ops::Range};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;
use log::info;
use crate::common::utils::IsNullOrEmpty;
use crate::{model::{self, Mesh}, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
            Ok(txt)
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            info!("Loading string from path: {:?}", path);
            let txt = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("Failed to read file {:?}: {:?}", path, e))?;
            Ok(txt)
        }
    }
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    match load_binary(file_name).await {
        Ok(data) => {
            let data = data.into_boxed_slice();
            texture::Texture::from_bytes(device, queue, &data, file_name)
        }
        Err(e) => {
            // Check if the error is a "file not found" or HTTP 404
            let is_not_found = e.downcast_ref::<std::io::Error>()
                .map(|io_err| io_err.kind() == std::io::ErrorKind::NotFound)
                .unwrap_or_else(|| {
                    // For WASM, check if it's a 404 error
                    if cfg!(target_arch = "wasm32") {
                        //Todo decide if request code should be moved under wasm decorator
                        e.downcast_ref::<reqwest::Error>()
                            .and_then(|req_err| req_err.status())
                            .map(|status| status == reqwest::StatusCode::NOT_FOUND)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                });

            if is_not_found {
                info!("Texture '{}' not found, using default white texture", file_name);
                // Safe to unwrap if [0xFF; 4] is always a valid 1x1 texture
                texture::Texture::from_bytes(
                    device, 
                    queue, 
                    &[0xFF, 0xFF, 0xFF, 0xFF], 
                    "white"
                ).map_err(|err| {
                    anyhow::anyhow!(
                        "BUG: Failed to create white fallback texture: {:?}", 
                        err
                    )
                })
            } else {
                // Propagate other errors (e.g., permissions, network issues)
                info!("Error loading texture '{}': {:?}", file_name, e);
                Err(e)
            }
        }
    }
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let response = reqwest::get(url).await?;
            
            // Handle HTTP errors explicitly
            if response.status().is_success() {
                let data = response.bytes().await?.to_vec();
                Ok(data)
            } else {
                Err(anyhow::anyhow!(
                    "HTTP error {} for {}", 
                    response.status(), 
                    file_name
                ))
            }
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            info!("Loading binary from path: {:?}", path);
            
            std::fs::read(&path)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to read file {:?}: {}", 
                        path, 
                        e
                    )
                })
        }
    }
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    // Log the contents of the folder
    let path = std::path::Path::new(env!("OUT_DIR")).join("res");
    info!("Contents of the folder: {:?}", std::fs::read_dir(&path)?.collect::<Vec<_>>());

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty(){
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [0.0, 0.0, 0.0],
                        }
                    }else{
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

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

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
    );
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
    ){
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

