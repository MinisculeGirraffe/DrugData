use entity::Product::*;
use sea_orm_migration::prelude::*;
pub struct Migration;

use anyhow;

use futures::future::join_all;
use log::info;

use std::io::{copy, Cursor};

use std::{fs, path::PathBuf};
use tempfile::{Builder, TempDir};

use sea_orm::{DbConn, EntityTrait};

const FDA_URL: &str = "https://www.fda.gov/media/89850/download";

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220618_162459_create_product_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let tmp_dir = Builder::new().prefix("fda").tempdir().unwrap();
        // create the table
        manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(Column::ApplNo).string_len(6).not_null())
                    .col(
                        ColumnDef::new(Column::ProductNo)
                            .string_len(6)
                            .not_null()
                    )
                    .col(ColumnDef::new(Column::Form).string())
                    .col(ColumnDef::new(Column::Strength).string())
                    .col(ColumnDef::new(Column::ReferenceDrug).integer())
                    .col(ColumnDef::new(Column::DrugName).string())
                    .col(ColumnDef::new(Column::ActiveIngredient).string())
                    .col(ColumnDef::new(Column::ReferenceStandard).integer())
                    .primary_key(Index::create().col(Column::ApplNo).col(Column::ProductNo))
                    .to_owned(),
            )
            .await?;

        let zip = download_zip(&tmp_dir).await.unwrap();
        let files = extract_zip(zip, &tmp_dir).await.unwrap();
        for file in files {
            match file.file_name().unwrap().to_str().unwrap() {
                "Products.txt" => load_data(file, db).await.unwrap(),
                &_ => continue,
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}

async fn download_zip(dir: &TempDir) -> anyhow::Result<PathBuf> {
    info!("Downloading file from {}", FDA_URL);
    let response = reqwest::get(FDA_URL).await?;
    // get content disposition header
    let cd = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .unwrap();

    // get filename from content disposition response header
    let file_name = actix_web::http::header::ContentDisposition::from_raw(cd)?
        .get_filename()
        .unwrap()
        .to_string();
    info!("Got filename {} from Content Disposition", file_name);

    //merge filename with process temp directory
    let file_path = dir.path().join(&file_name);
    //write binary from HTTP response to file
    let mut file = fs::File::create(&file_path)?;
    let mut content = Cursor::new(response.bytes().await?);
    info!("Saving file to {:?}", &file_path);
    copy(&mut content, &mut file)?;
    info!("File saved sucessfully");
    anyhow::Ok(file_path)
}

async fn extract_zip(file: PathBuf, dir: &TempDir) -> anyhow::Result<Vec<PathBuf>> {
    // array of extracted filepaths
    let mut files: Vec<PathBuf> = vec![];

    // open and parse zip file
    info!("Attempting to extract {:?}", &file);
    let zip = fs::File::open(file)?;
    let mut archive = zip::ZipArchive::new(zip)?;

    //iterate over files in zip
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        // skip if current index has no file name
        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        // join zip filename with temp directory
        let outpath = dir.path().join(file_path);
        if (*file.name()).ends_with('/') {
            info!("File {} extracted to \"{}\"", i, outpath.display());
            //create parent directory path if item is a folder
            fs::create_dir_all(&outpath).unwrap();
        } else {
            info!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            //write zip file to filesystem in temp directory
            let mut outfile = fs::File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
            files.push(outpath.clone());
        }
    }
    //return array of filepaths
    anyhow::Ok(files)
}

async fn load_data(path: PathBuf, db: &DbConn) -> anyhow::Result<()> {
    let mut results = vec![];
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_path(&path)?;
    info!("Starting import of file {:?}", &path);
    for result in rdr.deserialize() {
        let record: Model = result?;
        let active_model: ActiveModel = record.into();
        results.push(Entity::insert(active_model).exec(db));
    }

    let result = join_all(results).await;
    info!("Finished importing {} records", { result.len() });

    anyhow::Ok(())
}
