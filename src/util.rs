use rustfft::num_traits::Float;

pub fn bipolar_interpolation (
    a: f32,
    b: f32,
    distance: f32
) -> f32 {
    a * (1. - distance) + b * distance
}

pub fn rescale(
    source: &[f32],
    destination: &mut [f32],
    interpolation: impl Fn(f32, f32, f32) -> f32
) {
    if source.len() == destination.len() {
        destination.copy_from_slice(source);
        return;
    }

    let scale = destination.len() as f32 / source.len() as f32;

    println!("rescale start");

    destination[0] = source[0];
    for i in 1..destination.len() {
        let last = ((i - 1) as f32 / scale).ceil();
        let a = (i as f32 / scale).floor();
        let b = (i as f32 / scale).ceil();

        if a - last > 1. {
            let s = &source[last as usize..b as usize ];
            destination[i] = s[0];

            for x in s {
                destination[i] = if x > &destination[i]  {
                    *x
                } else {
                    destination[i]
                }
            }

        } else {
            destination[i] = interpolation(
                source[a as usize],
                source[b as usize],
                i as f32 / scale - a
            );
        }
        println!("{}", destination[i]);
        println!("{} {}", a, b);
    }
}