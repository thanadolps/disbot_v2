use image::{ImageBuffer, Pixel};
use slice_of_array::prelude::*;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct BrailleConfig {
    pub monospace: bool,
    pub invert: bool,
    pub threshold: u8,
}

impl Default for BrailleConfig {
    fn default() -> Self {
        BrailleConfig {
            monospace: false,
            invert: false,
            threshold: 125,
        }
    }
}

fn pattern_to_braille(pattern: [[bool; 2]; 4], config: &BrailleConfig) -> char {
    const SHIFT_CODE: [u8; 8] = [0, 3, 1, 4, 2, 5, 6, 7];

    let flatten = pattern.flat();
    let offset = flatten
        .iter()
        .zip(SHIFT_CODE.iter())
        .map(|(&v, sc)| (v as u32) << sc)
        .sum::<u32>();

    // TODO: replace with uncheck version
    if !config.monospace && offset == 0 {
        '⠄'
        // ' '
        // ' '
    } else {
        std::char::from_u32(0x2800 + offset).expect("to always be in range of valid unicode")
    }
}

// (x, y) = upper left coordinate
fn extract_pattern<P, C>(
    image: &ImageBuffer<P, C>,
    x: u32,
    y: u32,
    config: &BrailleConfig,
) -> [[bool; 2]; 4]
where
    P: Pixel<Subpixel = u8> + 'static,
    C: Deref<Target = [P::Subpixel]>,
{
    let mut buf = [[false; 2]; 4];

    let put_into_buf = |buf: &mut [[bool; 2]; 4], i, j| {
        let px = image.get_pixel(x + i, y + j);
        let val = px.to_luma().0[0];
        buf[j as usize][i as usize] = if !config.invert {
            val < config.threshold
        } else {
            val > config.threshold
        };
    };
    put_into_buf(&mut buf, 0, 0);
    put_into_buf(&mut buf, 0, 1);
    put_into_buf(&mut buf, 0, 2);
    put_into_buf(&mut buf, 0, 3);
    put_into_buf(&mut buf, 1, 0);
    put_into_buf(&mut buf, 1, 1);
    put_into_buf(&mut buf, 1, 2);
    put_into_buf(&mut buf, 1, 3);

    buf
}

// struct PatternRow<'a, P, C>
// where
//     P: Pixel<Subpixel = u8> + 'static,
//     C: Deref<Target = [P::Subpixel]>,
// {
//     image: &'a ImageBuffer<P, C>,
//     y: u32,
//     config: &'a BrailleConfig,

//     x: u32,
// }

// impl<'a, P, C> Iterator for PatternRow<'a, P, C>
// where
//     P: Pixel<Subpixel = u8> + 'static,
//     C: Deref<Target = [P::Subpixel]>,
// {
//     type Item = char;

//     fn next(&mut self) -> Option<Self::Item> {
//         let w = self.image.width() - 1;

//         if self.x >= w {
//             return None;
//         }

//         let pattern = extract_pattern(self.image, self.x, self.y, self.config);
//         let braille = pattern_to_braille(pattern, self.config);
//         self.x += 2;
//         Some(braille)
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let w = self.image.width() - 1;
//         let len = ((w - self.x) / 2) as usize;
//         (len, Some(len))
//     }
// }

fn extract_pattern_row<'a, P, C>(
    image: &'a ImageBuffer<P, C>,
    y: u32,
    config: &'a BrailleConfig,
) -> impl Iterator<Item = char> + 'a
where
    P: Pixel<Subpixel = u8> + 'static,
    C: Deref<Target = [P::Subpixel]>,
{
    let w = image.width() - 1;
    (0..w).step_by(2).map(move |x| (x, y)).map(move |(x, y)| {
        let pattern = extract_pattern(image, x, y, config);
        pattern_to_braille(pattern, config)
    })
    // PatternRow {
    //     image,
    //     y,
    //     config,
    //     x: 0,
    // }
}

// pub struct PatternIter<'a, P, C>
// where
//     P: Pixel<Subpixel = u8> + 'static,
//     C: Deref<Target = [P::Subpixel]> + 'static,
// {
//     image: &'a ImageBuffer<P, C>,
//     config: &'a BrailleConfig,
//     y: u32,
// }

// impl<'a, P, C> Iterator for PatternIter<'a, P, C>
// where
//     P: Pixel<Subpixel = u8> + 'static,
//     C: Deref<Target = [P::Subpixel]> + 'static,
// {
//     type Item = PatternRow<'a, P, C>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let y = self.y;
//         self.y += 4;
//         Some(extract_pattern_row(self.image, y, self.config))
//     }
// }

pub fn image_to_patterns<'a, P, C>(
    image: &'a ImageBuffer<P, C>,
    config: &'a BrailleConfig,
) -> impl Iterator<Item = impl Iterator<Item = char> + 'a> + 'a
where
    P: Pixel<Subpixel = u8> + 'static,
    C: Deref<Target = [P::Subpixel]> + 'static,
{
    let y = (0..image.height() - 3).step_by(4);
    y.map(move |y| extract_pattern_row(image, y, config))
    // PatternIter {
    //     image,
    //     config,
    //     y: 0,
    // }
}

// calculate image dimension that need to make output braille <= max_braille, will keep aspect ratio
pub fn calculate_image_size(old_dim: (u32, u32), max_braille: usize) -> (u32, u32) {
    let (w, h) = old_dim;
    let scaler = ((8 * max_braille) as f32 / (w * h) as f32).sqrt().min(4.0);

    let w2 = (scaler * w as f32) as u32;
    let h2 = (scaler * h as f32) as u32;

    (w2, h2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::{BufWriter, Write};

    #[test]
    fn ptb() {
        assert_eq!(
            pattern_to_braille([[false, false]; 4], &Default::default()),
            '⠀'
        );
        assert_eq!(
            pattern_to_braille(
                [[false, false]; 4],
                &BrailleConfig {
                    monospace: true,
                    threshold: 125,
                    invert: false
                }
            ),
            '⠄'
        );
        assert_eq!(
            pattern_to_braille([[true, true]; 4], &Default::default()),
            '⣿'
        );
        assert_eq!(
            pattern_to_braille([[false, true]; 4], &Default::default()),
            '⢸'
        );
        assert_eq!(
            pattern_to_braille([[true, false]; 4], &Default::default()),
            '⡇'
        );
        assert_eq!(
            pattern_to_braille(
                [[false, false], [true, false], [false, true], [false, true]],
                &Default::default()
            ),
            '⢢'
        );
    }

    fn itp() {
        let mut img = image::open("./sample_croped.jpg").unwrap().into_luma8();
        let (w, h) = img.dimensions();

        let config = BrailleConfig {
            monospace: false,
            threshold: 125,
            invert: true,
        };

        let target = 50000;
        let scaler = ((8 * target) as f32 / (w * h) as f32).sqrt().min(4.0);

        let w2 = (scaler * w as f32) as u32;
        let h2 = (scaler * h as f32) as u32;

        let mut img =
            image::imageops::resize(&img, w2, h2, image::imageops::FilterType::CatmullRom);

        if true {
            image::imageops::dither(&mut img, &image::imageops::BiLevel)
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./text.txt")
            .unwrap();
        let mut buffer = BufWriter::new(file);

        let mut buf = [0; 4];
        image_to_patterns(&img, &config).for_each(|row| {
            row.for_each(|val| {
                let v = val.encode_utf8(&mut buf);
                buffer.write_all(v.as_bytes()).unwrap();
            });
            buffer.write(b"\n").unwrap();
        });
    }
}
