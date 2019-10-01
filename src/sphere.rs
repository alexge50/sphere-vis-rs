#[derive(Clone)]
pub struct Sphere {
    pub vertices: Vec<f32>,
    pub indices: Vec<i32>,
}

impl Sphere {
    pub fn generate(
        radius: f32,
        ring_count: i32,
        sector_count: i32
    ) -> Sphere {
        return Sphere {
            vertices: generate_vertices(radius, ring_count, sector_count),
            indices: generate_indices(ring_count, sector_count)
        };
    }
}

fn generate_vertices(
    radius: f32,
    ring_count: i32,
    sector_count: i32
) -> Vec<f32> {

    let mut vertices = Vec::<f32>::with_capacity(
        (ring_count * sector_count * 3) as usize
    );

    use std::f32::consts::PI;

    let ring_step: f32 = 1. / (ring_count - 1) as f32;
    let sector_step: f32 = 1. / (sector_count - 1) as f32;

    for r in 0..ring_count{
        for s in 0..sector_count {
            let x = radius *
                (2. * PI * s as f32 * sector_step).cos() *
                (PI * r as f32 * ring_step).sin();
            let y = radius *
                (-PI / 2. + PI * r as f32 * ring_step).sin();
            let z = radius *
                (2. * PI * s as f32 * sector_step).sin() *
                (PI * r as f32 * sector_step).sin();

            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
        }
    }

    return vertices;
}

fn generate_indices(
    ring_count: i32,
    sector_count: i32
) -> Vec<i32> {

    let mut indices = Vec::<i32>::with_capacity(
        (ring_count * sector_count * 4) as usize
    );

    for r in 0..ring_count - 1 {
        for s in 0..sector_count - 1 {
            indices.push(r * sector_count + s);
            indices.push(r * sector_count + s + 1);
            indices.push((r + 1) * sector_count + s + 1);
            indices.push((r + 1) * sector_count + s);
        }
    }

    return indices;
}