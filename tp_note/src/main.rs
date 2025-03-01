use argh::FromArgs;
use image::{DynamicImage,ImageBuffer,RgbImage};
use image::error::ImageError;
use std::collections::HashMap;
use rand::Rng;


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
    Tramage(OptsTramage),
    Tramagebayer(OptsTramagebayer),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {
    /// première couleur
    #[argh(option)]
    couleur1: Option<String>,

    /// deuxième couleur
    #[argh(option)]
    couleur2: Option<String>,
}


#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage")]
/// Rendu de l’image par seuillage monochrome.
struct OptsTramage {

}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramagebayer")]
/// Rendu de l'image par matrice de Bayer
struct OptsTramagebayer {
    /// la taille de la matrice
    #[argh(option)]
    ordre: u32
}


fn calcul_luminosite(pixel: [u8; 3]) -> f32 {
    return 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
}


fn distance_euclidienne(c1: [u8; 3], c2: [u8; 3]) -> f32 {
    let r_diff = (c1[0] as f32 - c2[0] as f32).powi(2);
    let g_diff = (c1[1] as f32 - c2[1] as f32).powi(2);
    let b_diff = (c1[2] as f32 - c2[2] as f32).powi(2);
    
    (r_diff + g_diff + b_diff).sqrt()
}

fn couleur_plus_proche(pixel: [u8; 3], palette: &[[u8; 3]]) -> [u8; 3] {
    *palette.iter()
        .min_by(|&&c1, &&c2|
            distance_euclidienne(pixel, c1)
                .partial_cmp(&distance_euclidienne(pixel, c2))
                .unwrap()
        )
        .unwrap()
}

fn palette_reduite(img: &mut RgbImage, n_couleurs: usize) {
    // Palette des couleurs de base (NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA)
    let palette: Vec<[u8; 3]> = vec![
        [0, 0, 0], // Noir
        [255, 255, 255], // Blanc
        [255, 0, 0], // Rouge
        [0, 255, 0], // Vert
        [0, 0, 255], // Bleu
        [255, 255, 0], // Jaune
        [0, 255, 255], // Cyan
        [255, 0, 255], // Magenta
    ];

    // Limiter la palette à n_couleurs
    let palette = &palette[..n_couleurs];

    // Parcourir chaque pixel et trouver la couleur la plus proche dans la palette
    for pixel in img.pixels_mut() {
        let couleur_pixel = pixel.0; // RGB de ce pixel
        let couleur_proche = couleur_plus_proche(couleur_pixel, palette);
        *pixel = image::Rgb(couleur_proche);
    }
}

fn tramage_aleatoire(img: &mut RgbImage) {
    let mut rng = rand::thread_rng();

    for pixel in img.pixels_mut() {
        let luminosite = calcul_luminosite(pixel.0);
        let seuil_aleatoire: f32 = rng.gen(); // Génère un nombre aléatoire entre 0 et 1

        *pixel = if luminosite / 255.0 > seuil_aleatoire {
            image::Rgb([255, 255, 255]) // Blanc
        } else {
            image::Rgb([0, 0, 0]) // Noir
        };
    }
}

fn generer_matrice_bayer(n: u32) -> Vec<Vec<u8>> {
    if n == 0 {
        return vec![vec![0]];
    }
    let taille = 2usize.pow(n);
    let taille_precedente = taille / 2;
    let matrice_precedente = generer_matrice_bayer(n - 1);
    let mut matrice = vec![vec![0; taille]; taille];
    for i in 0..taille_precedente {
        for j in 0..taille_precedente {
            let valeur = matrice_precedente[i][j];
            matrice[i][j] = 4 * valeur;
            matrice[i][j + taille_precedente] = 4 * valeur + 2;
            matrice[i + taille_precedente][j] = 4 * valeur + 3;
            matrice[i + taille_precedente][j + taille_precedente] = 4 * valeur + 1;
        }
    } 
    matrice
 }
 

fn tramage_bayer(img: &mut RgbImage, n: u32) {
    let matrice_bayer = generer_matrice_bayer(n);
    let taille = matrice_bayer.len() as u32;
   
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let luminosite = calcul_luminosite(pixel.0) / 255.0;
        let seuil = matrice_bayer[(y % taille) as usize][(x % taille) as usize] as f32 / (taille * taille) as f32;
 
 
        *pixel = if luminosite > seuil {
            image::Rgb([255, 255, 255]) // Blanc
        } else {
            image::Rgb([0, 0, 0]) // Noir
        };
    }
 }
 
fn diffusion_erreur(img: &mut RgbImage) {
    let (width, height) = img.dimensions();
    let mut erreurs = vec![vec![0.0; width as usize]; height as usize];
    
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel_mut(x, y);
            let luminosite = calcul_luminosite(pixel.0) / 255.0;
            let nouvelle_luminosite = (luminosite + erreurs[y as usize][x as usize]).clamp(0.0, 1.0);
            
            let nouvelle_couleur = if nouvelle_luminosite > 0.5 { 255 } else { 0 };
            let erreur = nouvelle_luminosite - (nouvelle_couleur as f32 / 255.0);
            *pixel = image::Rgb([nouvelle_couleur, nouvelle_couleur, nouvelle_couleur]);
            
            if x + 1 < width {
                erreurs[y as usize][(x + 1) as usize] += erreur * 0.5;
            }
            if y + 1 < height {
                erreurs[(y + 1) as usize][x as usize] += erreur * 0.5;
            }
        }
    }
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
            let couleur1 = if let Some(c1) = &opts.couleur1 {
                couleurs.get(c1.to_lowercase().as_str()).unwrap_or(&[0, 0, 0]) // Noir si non trouvé
            } else {
                &[0, 0, 0] // Noir si non défini
            };
        
            // Gestion de couleur2 (Blanc par défaut)
            let couleur2 = if let Some(c2) = &opts.couleur2 {
                couleurs.get(c2.to_lowercase().as_str()).unwrap_or(&[255, 255, 255]) // Blanc si non trouvé
            } else {
                &[255, 255, 255] // Blanc si non défini
            };
        
            for pixel in rgb8_img.pixels_mut() {
                let luminosite = calcul_luminosite(pixel.0);
                *pixel = if luminosite > 128.0 {
                    image::Rgb(*couleur2) // Lumineux → Couleur claire (blanc par défaut)
                } else {
                    image::Rgb(*couleur1) // Sombre → Couleur sombre (noir par défaut)
                };
            }
        }
        Mode::Palette(opts) => {
            if opts.n_couleurs == 0 {
                eprintln!("Erreur : Vous devez spécifier au moins 1 couleur pour la palette.");
                std::process::exit(1);
            }
            palette_reduite(&mut rgb8_img, opts.n_couleurs);
        }
        Mode::Tramage(_) => {
            diffusion_erreur(&mut rgb8_img);
            // tramage_aleatoire(&mut rgb8_img);
        }
        Mode::Tramagebayer(opts) => {
            tramage_bayer(&mut rgb8_img, opts.ordre);
        }
    }
   
    if let Some(output) = args.output {
        rgb8_img.save(output)?;
    }

    return Ok(());
}
