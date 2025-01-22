use argh::FromArgs;
use image::{DynamicImage,ImageBuffer,RgbImage};
use image::error::ImageError;
use std::collections::HashMap;


#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {

    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {
    /// première couleur
    #[argh(option)]
    couleur1: String,

    /// deuxième couleur
    #[argh(option)]
    couleur2: String,
}



#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

fn calcul_luminosite(pixel: [u8; 3]) -> f32 {
    return 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
}

fn main() -> Result<(), ImageError> {
    // let args: DitherArgs = argh::from_env();
    // let path_in = args.input;
    // Ok(())

    //question 2
    // let img =  image::io::Reader::open("./image/myimage.jpg")?.decode()?;
    // let mut rgb8_img: RgbImage = img.to_rgb8();

    //question 3
    // rgb8_img.save("./image/ballon2.png")?;

    //question 8
    let args: DitherArgs = argh::from_env();
    let img = image::io::Reader::open(&args.input)?.decode()?;
    let mut rgb8_img: RgbImage = img.to_rgb8();
    let couleurs: HashMap<&str, [u8; 3]> = [
        ("noir", [0, 0, 0]),
        ("blanc", [255, 255, 255]),
        ("rouge", [255, 0, 0]),
        ("vert", [0, 255, 0]),
        ("bleu", [0, 0, 255]),
        ("jaune", [255, 255, 0]),
        ("cyan", [0, 255, 255]),
        ("magenta", [255, 0, 255]),
    ]
    .iter()
    .cloned()
    .collect();


    //question 5
    // for (x, y, pixel) in rgb8_img.enumerate_pixels_mut() {
    //     // if (x+y) % 2 == 0 {

    //     //     *pixel = image::Rgb([255, 255, 255]);
    //     // }

    //     // question 8
    //     let luminosité= calcul_luminosite(pixel.0);
    //     if luminosité > 128.0 {
    //         *pixel = image::Rgb([255, 255, 255]);
    //     } else {
    //         *pixel = image::Rgb([0, 0, 0]);
    //     }
        
    // }
    // rgb8_img.save("./image/question_8.png")?;

    

    // //question 4 
    // let pixel = rgb8_img.get_pixel(32, 52);
    // println!("Couleur du pixel (32,52) : {:?}", pixel);


    //question 8
    match &args.mode {
        Mode::Seuil(opts) => {
            let couleur1 = couleurs.get(opts.couleur1.to_lowercase().as_str()).unwrap_or(&[0, 0, 0]);
            let couleur2 = couleurs.get(opts.couleur2.to_lowercase().as_str()).unwrap_or(&[255, 255, 255]);
            
            for pixel in rgb8_img.pixels_mut() {
                let luminosite = calcul_luminosite(pixel.0);
                *pixel = if luminosite > 128.0 {
                    image::Rgb(*couleur2)
                } else {
                    image::Rgb(*couleur1)
                };
            }
        }
        Mode::Palette(opts) => {
            println!("Mode palette avec {} couleurs", opts.n_couleurs);
            // Implémentation du mode palette si nécessaire
        }
    }
    if let Some(output) = args.output {
        rgb8_img.save(output)?;
    }

    return Ok(());
}
