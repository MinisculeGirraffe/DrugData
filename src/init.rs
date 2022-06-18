use anyhow;

use entity::{Applications, Product};

use futures::future::join_all;
use log::info;
use serde::de::DeserializeOwned;

use std::io::{copy, Cursor};

use std::{fs, path::PathBuf};
use tempfile::{Builder, TempDir};

use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, DbConn, EntityTrait,
    ModelTrait, Schema,
};

const FDA_URL: &str = "https://www.fda.gov/media/89850/download";

pub async fn setup() -> anyhow::Result<DatabaseConnection> {
    let db_url = "sqlite::memory:";
    let conn = sea_orm::Database::connect(db_url).await.unwrap();
    let tmp_dir = Builder::new().prefix("fda").tempdir()?;

    let zip = download_zip(&tmp_dir).await?;
    let files = extract_zip(zip, &tmp_dir).await?;

    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();

        match file_name {
            "Products.txt" => {
                setup_table::<Product::Model, Product::ActiveModel, Product::Entity>(
                    file,
                    &conn,
                    Product::Entity,
                )
                .await?
            }
            "Applications.txt" => {
                setup_table::<Applications::Model, Applications::ActiveModel, Applications::Entity>(
                    file,
                    &conn,
                    Applications::Entity,
                )
                .await?
            }
            &_ => continue,
        }
    }
    anyhow::Ok(conn)
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

async fn setup_table<Model, ActiveModel, Entity>(
    path: PathBuf,
    db: &DbConn,
    entity: Entity,
) -> anyhow::Result<()>
where
    Model: ModelTrait + DeserializeOwned + Into<ActiveModel>,
    ActiveModel: ActiveModelTrait<Entity = Entity>,
    Entity: EntityTrait,
{
    let mut results = vec![];
    let schema = Schema::new(DbBackend::Sqlite);
    info!("Creating Table in memory for {}", entity.as_str());
    let stmt = schema.create_table_from_entity(entity);
    let sql = db.get_database_backend().build(&stmt);
    db.execute(sql).await?;
    info!("Table creation sucessful for {}", entity.as_str());

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
