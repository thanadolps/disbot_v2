use color_eyre::eyre::Result;
use genawaiter::{
    stack::{let_gen, let_gen_using, producer_fn, Co},
    yield_,
};
use image::{DynamicImage, GenericImageView, Luma, Pixel};
use itertools::Itertools;
use poise::{command, serenity_prelude::Attachment};

use crate::{braille, Context, DISCORD_MESSAGE_LIMIT, DISCORD_WIDTH_LIMIT};

const N_CHAR_IN_ROW: usize = DISCORD_WIDTH_LIMIT;
const ROW_PER_MESSAGE: usize = (DISCORD_MESSAGE_LIMIT / N_CHAR_IN_ROW) as usize;

/// Convert a provided image into text (braille unicode)
#[command(prefix_command, slash_command)]
pub async fn unicode(
    ctx: Context<'_>,
    image: Attachment,
    invert: bool,
    monospace: bool,
) -> Result<()> {
    unicode_inner(ctx, image, invert, monospace).await
}

async fn unicode_inner(
    ctx: Context<'_>,
    image: Attachment,
    invert: bool,
    monospace: bool,
) -> Result<()> {
    if image.dimensions().is_none() {
        ctx.reply("Must have an image attachment").await?;
        return Ok(());
    }
    let image_data = image.download().await?;
    let image = image::load_from_memory(&image_data)?;

    // Resize image to fit into discord message limit
    // TODO: allow resizing by char width & using multiple message to bypass character limit
    let (w, h) = image.dimensions();
    let w2 = 2 * (N_CHAR_IN_ROW as u32 - 1); // one braille = 2 px width, -1 from newline
    let h2 = (h * w2) / w; // maintain aspect ratio

    let mut image = image
        .resize(w2, h2, image::imageops::FilterType::CatmullRom)
        .to_luma8();

    image::imageops::dither(&mut image, &image::imageops::BiLevel);

    // Convert image to braille patterns
    let config = braille::BrailleConfig {
        invert,
        monospace,
        ..Default::default()
    };
    let mut pattern_iter = braille::image_to_patterns(&image, &config);

    // Produce messages
    loop {
        let mut buf = String::with_capacity(DISCORD_MESSAGE_LIMIT);
        pattern_iter.by_ref().take(ROW_PER_MESSAGE).for_each(|row| {
            buf.extend(row);
            buf.push('\n');
        });
        if buf.is_empty() {
            break;
        }

        if buf.chars().count() > DISCORD_MESSAGE_LIMIT {
            eprintln!(
                "[WARN] Message too long: {} > {}",
                buf.chars().count(),
                DISCORD_MESSAGE_LIMIT
            );
            eprintln!("{}", buf);
        }
        ctx.say(buf).await?;
    }

    Ok(())
}

// async fn unicode_message_producer<'a>(
//     co: Co<'_, String>,
//     image: DynamicImage,
//     invert: bool,
//     monospace: bool,
// ) -> Result<impl IntoIterator<Item = String>> {
//     // TODO: make a more solid solution
//     // let arg_set = args
//     //     .iter()
//     //     .take(10)
//     //     .filter_map(|x| x.ok())
//     //     .collect::<HashSet<String>>();
//     // let invert = arg_set.contains("invert");
//     // let monospace = !arg_set.contains("pad");

//     // let (w2, h2) = braille::calculate_image_size(image.dimensions(), 2000);

//     // TODO: allow resizing by char width & using multiple message to bypass character limit
//     let (w, h) = image.dimensions();
//     let w2 = 2 * N_CHAR_IN_ROW as u32; // one braille = 2 px width
//     let h2 = (h * w2) / w; // maintain aspect ratio

//     let mut image = image
//         .resize(w2, h2, image::imageops::FilterType::CatmullRom)
//         .to_luma8();

//     image::imageops::dither(&mut image, &image::imageops::BiLevel);

//     let config = braille::BrailleConfig {
//         monospace,
//         invert,
//         ..Default::default()
//     };
//     dbg!(&config);

//     let pattern_iter = braille::image_to_patterns(&image, &config);

//     // let message_iter = pattern_iter
//     //     .chunks(ROW_PER_MESSAGE)
//     //     .into_iter()
//     //     .map(move |row| {
//     //         let mut buf = String::new();
//     //         row.for_each(|row| {
//     //             buf.extend(row);
//     //             buf.push('\n');
//     //         });
//     //         buf.clone()
//     //     });

//     let_gen_using!(message_iter, async move |co| {
//         co.yield_("".to_string()).await
//     });

//     Ok(message_iter)

//     // Ok(a)

//     // loop {
//     //     pattern_iter.by_ref().take(ROW_PER_MESSAGE).for_each(|row| {
//     //         buf.extend(row);
//     //         buf.push('\n');
//     //     });

//     //     if buf.is_empty() {
//     //         break;
//     //     }
//     //     buf.clear();
//     // }

//     // Ok(())
// }

// mod iter {
//     use super::ROW_PER_MESSAGE;
//     use crate::braille::PatternIter;
//     use image::Pixel;
//     use std::ops::Deref;

//     pub(super) struct BrailleMessageIter<'a, P, C>
//     where
//         P: Pixel<Subpixel = u8> + 'static,
//         C: Deref<Target = [P::Subpixel]> + 'static,
//     {
//         pattern_iter: PatternIter<'a, P, C>,
//         message_buffer: String,
//     }

//     impl<'a, P, C> BrailleMessageIter<'a, P, C>
//     where
//         P: Pixel<Subpixel = u8> + 'static,
//         C: Deref<Target = [P::Subpixel]> + 'static,
//     {
//         pub(super) fn new(pattern_iter: PatternIter<'a, P, C>) -> Self {
//             Self {
//                 pattern_iter,
//                 message_buffer: String::new(),
//             }
//         }
//     }

//     // impl<'a, P, C> Iterator for BrailleMessageIter<'a, P, C>
//     // where
//     //     P: Pixel<Subpixel = u8> + 'static,
//     //     C: Deref<Target = [P::Subpixel]> + 'static,
//     // {
//     //     type Item = &'a str;
//     //     fn next<'s>(&'s mut self) -> Option<Self::Item> {
//     //         for row in self.pattern_iter.by_ref().take(ROW_PER_MESSAGE) {
//     //             self.message_buffer.extend(row);
//     //             self.message_buffer.push('\n');
//     //         }

//     //         if self.message_buffer.is_empty() {
//     //             None
//     //         } else {
//     //             Some(self.message_buffer.as_str())
//     //         }
//     //     }
//     // }
// }
