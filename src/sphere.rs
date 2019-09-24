pub struct Sphere {
    pub vertices: Vec<f32>,
    pub indices: Vec<i32>,
}

impl Sphere {
    pub fn generate(
        radius: f32,
        stack_count: i32,
        sector_count: i32
    ) -> Sphere {
        return Sphere {
            vertices: generate_vertices(radius, stack_count, sector_count),
            indices: generate_indices(stack_count, sector_count)
        };
    }
}

fn generate_vertices(
    radius: f32,
    stack_count: i32,
    sector_count: i32
) -> Vec<f32> {

    let mut vertices = Vec::<f32>::with_capacity(
        (stack_count * sector_count * 3) as usize
    );

    use std::f64::consts::PI;

    let sector_step: f32 = (2. * PI / (stack_count - 1) as f64) as f32;
    let stack_step: f32 = (PI / (sector_count - 1) as f64) as f32;

    for i in 0..stack_count {
        let stack_angle: f32 = (PI as f32 / 2. - i as f32 * stack_step);
        let z = radius * stack_angle.sin();
        let xy = radius * stack_angle.cos();

        for j in 0..sector_count {
            let sector_angle = j as f32 * sector_step;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
        }
    }

    return vertices;
}

fn generate_indices(
    stack_count: i32,
    sector_count: i32
) -> Vec<i32> {

    let mut indices = Vec::<i32>::with_capacity(
        (stack_count * sector_count * 6) as usize
    );

    for i in 0..stack_count {
        let mut k1 = i  * (sector_count + 1);
        let mut k2 = k1 + sector_count + 1;

        for j in 0..sector_count {
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            if i != stack_count - 1 {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    return indices;
}