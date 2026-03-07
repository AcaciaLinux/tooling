use std::{env::args, fs::File, path::PathBuf};

use tooling::{
    error::ALResult,
    odb::{ObjectDatabase, driver::FsODBDriver},
};

fn main() {
    if let Err(e) = run() {
        println!("{}", e.backtrace_string())
    }
}

fn run() -> ALResult<()> {
    let args: Vec<String> = args().collect();
    let file = PathBuf::from(&args[1]);
    println!("Hello, world!");

    let mut src_file = File::open(&file).unwrap();
    let odb = ObjectDatabase::new(Box::new(FsODBDriver::new(PathBuf::from("./odb"))));
    let oid = odb.insert(&mut src_file)?;
    println!("{}", oid.1);

    /*
    let mut object_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("o.aobj")
        .unwrap();

    let mut src_file = File::open(&file).unwrap();

    // Reserve the oid data
    object_file.write(&[0u8; 33]).unwrap();
    let oid = ObjectID::from_sha256_stream(&mut src_file, &mut object_file)?.1;
    object_file.seek(SeekFrom::Start(0)).unwrap();
    object_file.write(&oid.id()).unwrap();

    let oid = ObjectID::from_sha256_file(&file)?.1;
    println!("{oid}");

    let mut f = file_create(&PathBuf::from("hash.bin"))?;
    let _ = f.write(&oid.id()).ok();
    let path = oid.to_path(2);

    let root = current_dir().unwrap().join("odb");
    println!("{}", root.join(path).str_lossy());
    */
    Ok(())
}
