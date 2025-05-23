use color_name::Color;
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use image::io::Reader as ImageReader;
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use regex::Regex;
use rusqlite::Connection;
use rusttype::{Font, Scale};
use std::{
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use crate::config;
use crate::db;
use crate::mail::{send_epubs, Attachment};

fn generate_cover(
    volume_title: &str,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("src/assets/cover.png")
        .unwrap()
        .decode()
        .unwrap();

    let font = Vec::from(include_bytes!("font/RobotoSlab-VariableFont_wght.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    draw_text_mut(
        &mut img,
        Rgba([255, 255, 60, 255]),
        15,
        112,
        Scale::uniform(30.0),
        &font,
        volume_title,
    );
    std::fs::create_dir_all(output_dir)?;
    let path = output_dir.join(format!("{}.png", &volume_title));
    img.save(&path)?;
    Ok(path)
}

fn load_stylesheet() -> String {
    let mut file = std::fs::File::open("src/assets/style.css").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

fn strip_chapter_colour(chapter_data: &str) -> String {
    let re = Regex::new(r#"<span style="color:\s*(#......).*?">(.*?)</span>"#).unwrap();
    re.replace_all(chapter_data, |captures: &regex::Captures| {
        let colour_arr = hex::decode(&captures[1][1..]).unwrap();
        let name = Color::similar([colour_arr[0], colour_arr[1], colour_arr[2]]);
        format!(
            "<span>&lt;{a}|{b}|{a}&gt;</span>",
            a = name,
            b = &captures[2]
        )
    })
    .to_string()
}

fn replace_mrsha_write(chapter_data: &str) -> String {
    let re = Regex::new(r#"<span.*?mrsha-write.*?>(.*?)</span>"#).unwrap();
    re.replace_all(chapter_data, |captures: &regex::Captures| {
        format!("<em>{}</em>", &captures[1])
    })
    .to_string()
}

fn generate_chapter(
    db_conn: &Connection,
    chapter: &db::Chapter,
    output_dir: &Path,
    strip_colour: bool,
) -> Result<Attachment, Box<dyn std::error::Error>> {
    let mut output = Vec::<u8>::new();
    std::fs::create_dir_all(output_dir.join("individual"))?;

    let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;
    epub.metadata("author", "pirate aba")?;
    epub.metadata("lang", "en")?;
    epub.metadata("title", &chapter.name)?;
    epub.metadata("generator", "rsauvehoover/wandering-inn-scraper")?;

    let cover_img = generate_cover(
        &format!("Chapter {}", chapter.name),
        &output_dir.join("..").join("covers"),
    );
    let img_file = ImageReader::open(cover_img?)?.decode()?;
    let mut img_bytes = Vec::new();
    img_file.write_to(
        &mut Cursor::new(&mut img_bytes),
        image::ImageOutputFormat::Png,
    )?;
    epub.add_cover_image(
        output_dir.join(format!("{}({}).png", chapter.id, chapter.name)),
        img_bytes.as_slice(),
        "image/png",
    )?;
    epub.stylesheet(load_stylesheet().as_bytes())?;

    let mut raw_data = replace_mrsha_write(&db::get_chapter_data(db_conn, chapter.id)?);
    if strip_colour {
        raw_data = strip_chapter_colour(&raw_data);
    }
    epub.add_content(
        EpubContent::new(
            format!("{}({}).xhtml", &chapter.id, &chapter.name),
            raw_data.as_bytes(),
        )
        .title(&chapter.name),
    )?;

    epub.generate(&mut output)?;

    let filename = format!("{}({}).epub", &chapter.id, &chapter.name);

    let mut file = std::fs::File::create(output_dir.join("individual").join(&filename))?;
    file.write_all(&output)?;
    Ok(Attachment {
        filename,
        mime: String::from("application/epub+zip"),
        bytes: output,
    })
}

fn generate_chapters(
    db_conn: &Connection,
    chapters: &Vec<db::Chapter>,
    output_dir: &Path,
    strip_colour: bool,
) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir.join("combined"))?;

    if chapters.is_empty() {
        return Ok(Vec::<Attachment>::default());
    }

    let mut combined_output = Vec::<u8>::new();
    let last_chapter = chapters.last().unwrap();
    let mut combined_epub = EpubBuilder::new(ZipLibrary::new()?)?;
    combined_epub.metadata("author", "pirate aba")?;
    combined_epub.metadata("lang", "en")?;
    combined_epub.metadata(
        "title",
        format!(
            "The Wandering Inn Chapters {}-{}",
            chapters[0].name, last_chapter.name
        ),
    )?;
    combined_epub.metadata("generator", "rsauvehoover/wandering-inn-scraper")?;
    combined_epub.stylesheet(load_stylesheet().as_bytes())?;

    let cover_img = generate_cover(
        &format!("Chapters {}-{}", chapters[0].name, last_chapter.name),
        &output_dir.join("..").join("covers"),
    );
    let img_file = ImageReader::open(cover_img?)?.decode()?;
    let mut img_bytes = Vec::new();
    img_file.write_to(
        &mut Cursor::new(&mut img_bytes),
        image::ImageOutputFormat::Png,
    )?;
    combined_epub.add_cover_image(
        output_dir.join(format!(
            "{}({})-{}({}).png",
            chapters[0].id, chapters[0].name, last_chapter.id, last_chapter.name
        )),
        img_bytes.as_slice(),
        "image/png",
    )?;
    combined_epub.inline_toc();

    let mut attachments = Vec::<Attachment>::new();

    for chapter in chapters {
        let mut raw_data = replace_mrsha_write(&db::get_chapter_data(db_conn, chapter.id)?);
        if strip_colour {
            raw_data = strip_chapter_colour(&raw_data);
        }
        combined_epub.add_content(
            EpubContent::new(
                format!("{}({}).xhtml", chapter.id, chapter.name),
                raw_data.as_bytes(),
            )
            .title(&chapter.name),
        )?;
        attachments.push(generate_chapter(
            db_conn,
            chapter,
            output_dir,
            strip_colour,
        )?);
        db::update_generated_chapter(db_conn, chapter.id, false)?;
    }

    combined_epub.generate(&mut combined_output)?;

    let mut file = std::fs::File::create(output_dir.join("combined").join(format!(
        "{}({})-{}({}).epub",
        chapters[0].id, chapters[0].name, last_chapter.id, last_chapter.name
    )))?;
    file.write_all(&combined_output)?;
    Ok(attachments)
}

fn generate_volume(
    db_conn: &Connection,
    volume: &db::Volume,
    chapters: &Vec<db::Chapter>,
    output_dir: &Path,
    strip_colour: bool,
) -> Result<Attachment, Box<dyn std::error::Error>> {
    let mut output = Vec::<u8>::new();

    let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;
    epub.metadata("author", "pirate aba")?;
    epub.metadata("lang", "en")?;
    epub.metadata("title", format!("The Wandering Inn {}", &volume.name))?;
    epub.metadata("generator", "rsauvehoover/wandering-inn-scraper")?;
    epub.stylesheet(load_stylesheet().as_bytes())?;

    let cover_img = generate_cover(&volume.name, &output_dir.join("..").join("covers"));
    let img_file = ImageReader::open(cover_img?)?.decode()?;
    let mut img_bytes = Vec::new();
    img_file.write_to(
        &mut Cursor::new(&mut img_bytes),
        image::ImageOutputFormat::Png,
    )?;
    epub.add_cover_image(
        output_dir.join(format!("{}.png", &volume.name)),
        img_bytes.as_slice(),
        "image/png",
    )?;

    epub.inline_toc();

    for chapter in chapters {
        let mut raw_data = replace_mrsha_write(&db::get_chapter_data(db_conn, chapter.id)?);
        if strip_colour {
            raw_data = strip_chapter_colour(&raw_data);
        }
        epub.add_content(
            EpubContent::new(
                format!("{}({}).xhtml", chapter.id, chapter.name),
                raw_data.as_bytes(),
            )
            .title(&chapter.name),
        )?;
    }

    epub.generate(&mut output)?;

    std::fs::create_dir_all(output_dir)?;

    let filename = format!("{}.epub", volume.name);

    let mut file = std::fs::File::create(output_dir.join(format!("{}.epub", volume.name)))?;
    file.write_all(&output)?;

    Ok(Attachment {
        filename,
        mime: String::from("application/epub+zip"),
        bytes: output,
    })
}

pub async fn generate_epubs(
    db_conn: &Connection,
    build_dir: &Path,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vols = Vec::<Attachment>::new();
    let mut vols_stripped = Vec::<Attachment>::new();
    let mut chaps = Vec::<Attachment>::new();
    let mut chaps_stripped = Vec::<Attachment>::new();

    if config.epub_gen.volumes {
        let volumes = db::get_volumes_to_regenerate(db_conn)?;

        if volumes.is_empty() {
            println!("No volumes to generate");
        } else {
            println!("Generating epubs for {} volumes", volumes.len());
        }

        for volume in volumes {
            println!("Generating epub for {}", volume.name);
            let chapters = db::get_chapters_by_volume(db_conn, volume.id)?;
            if config.epub_gen.strip_colour {
                vols.push(generate_volume(
                    db_conn,
                    &volume,
                    &chapters,
                    &build_dir.join("volumes_stripped_colour"),
                    true,
                )?);
            }
            vols_stripped.push(generate_volume(
                db_conn,
                &volume,
                &chapters,
                &build_dir.join("volumes"),
                false,
            )?);
            db::update_generated_volume(db_conn, volume.id, false)?;
        }
    } else {
        println!("Skipping volume generation");
    }

    if config.epub_gen.chapters {
        let chapters = db::get_chapters_to_regenerate(db_conn)?;
        if chapters.is_empty() {
            println!("No chapters to generate");
            return Ok(());
        }
        println!("Generating epubs for {} chapters", chapters.len());
        if config.epub_gen.strip_colour {
            chaps_stripped = generate_chapters(
                db_conn,
                &chapters,
                &build_dir.join("chapters_stripped_colour"),
                true,
            )?;
        }
        chaps = generate_chapters(db_conn, &chapters, &build_dir.join("chapters"), false)?;
    } else {
        println!("Skipping chapter generation");
    }

    send_epubs(&config.mail, &vols, &vols_stripped, &chaps, &chaps_stripped).await;

    Ok(())
}
