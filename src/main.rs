use actix_web::middleware::Logger;
use anyhow;
use entity::{Applications, Product};
use env_logger::Env;
use futures::future::join_all;
use serde::de::DeserializeOwned;

use std::io::{copy, Cursor};

use std::{fs, path::PathBuf};
use tempfile::{Builder, TempDir};

use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, DbConn,
    EntityTrait, ModelTrait, QueryFilter, Schema,
};

const FDA_URL: &str = "https://www.fda.gov/media/89850/download";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let db_url = "sqlite::memory:";
    let conn = sea_orm::Database::connect(db_url).await.unwrap();
    let res = setup_schema(&conn).await?;
    println!("{:?}", &res);
    let tmp_dir = Builder::new().prefix("fda").tempdir()?;

    println!("Hello, world!");
    let zip = download_zip(&tmp_dir).await?;
    let files = extract_zip(zip, &tmp_dir).await?;

    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();

        match file_name {
            "Products.txt" => {
                load_tsv::<Product::Model, Product::ActiveModel, Product::Entity>(file, &conn)
                    .await?
            }
            "Applications.txt" => {
                load_tsv::<Applications::Model, Applications::ActiveModel, Applications::Entity>(
                    file, &conn,
                )
                .await?
            }
            &_ => continue,
        }
    }

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    anyhow::Ok(
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(conn.clone()))
                .service(index)
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?,
    )
}

#[get("/drug/{name}")]
async fn index(
    db: web::Data<DatabaseConnection>,
    name: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = db.as_ref();
    let results = Product::Entity::find()
        .filter(Product::Column::DrugName.contains(name.as_str()))
        .all(conn)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(results))
}

async fn download_zip(dir: &TempDir) -> anyhow::Result<PathBuf> {
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
    println!("file to download: '{}'", &file_name);

    //merge filename with process temp directory
    let file_path = dir.path().join(file_name);
    println!("will be located under: '{:?}'", &file_path);

    //write binary from HTTP response to file
    let mut file = fs::File::create(&file_path)?;
    let mut content = Cursor::new(response.bytes().await?);

    copy(&mut content, &mut file)?;

    println!("Finished writing {:?}", &file_path);
    anyhow::Ok(file_path)
}

async fn extract_zip(file: PathBuf, dir: &TempDir) -> anyhow::Result<Vec<PathBuf>> {
    // array of extracted filepaths
    let mut files: Vec<PathBuf> = vec![];

    // open and parse zip file
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
        print!("{}", outpath.display());
        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            //create parent directory path if item is a folder
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
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

async fn setup_schema(db: &DbConn) -> anyhow::Result<()> {
    let schema = Schema::new(DbBackend::Sqlite);
    let mut statements = vec![];
    statements.push(schema.create_table_from_entity(Product::Entity));
    statements.push(schema.create_table_from_entity(Applications::Entity));
    for stmt in statements {
        let sql = db.get_database_backend().build(&stmt);
        println!("{:?}", &sql.sql);
        db.execute(sql).await?;
    }
    anyhow::Ok(())
}

async fn load_tsv<T, U, E>(path: PathBuf, db: &DbConn) -> anyhow::Result<()>
where
    T: ModelTrait + DeserializeOwned + Into<U>,
    U: ActiveModelTrait<Entity = E>,
    E: EntityTrait,
{
    let mut results = vec![];
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_path(path)?;

    for result in rdr.deserialize() {
        let record: T = result?;
        let active_model: U = record.into();
        results.push(E::insert(active_model).exec(db));
    }

    let res = join_all(results).await;
    println!("sucessfully inserted {} rows", res.len());
    anyhow::Ok(())
}
