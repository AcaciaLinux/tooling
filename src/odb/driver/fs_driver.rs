use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use xz::write::XzEncoder;

use crate::{
    fs::util::{file_create_rw, mkdir_p, rename},
    odb::{ObjectCompression, ObjectID, driver::ODBDriver},
    util::{
        io::{DynReadWrapper, DynWriteWrapper},
        uid::uuid_v4_str,
    },
};

pub struct FsODBDriver {
    root: PathBuf,
}

impl FsODBDriver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn get_root(&self) -> &Path {
        &self.root
    }

    fn temp_file(&self) -> PathBuf {
        self.root.join("tmp").join(uuid_v4_str())
    }
}

impl ODBDriver for FsODBDriver {
    fn insert(
        &self,
        stream: &mut dyn Read,
        compression: ObjectCompression,
    ) -> crate::error::ALResult<(usize, crate::odb::ObjectID)> {
        let tmp_file = self.temp_file();
        mkdir_p(tmp_file.parent().unwrap())?;

        let mut file = file_create_rw(&tmp_file)?;
        file.write_all("AOBJ".as_bytes()).unwrap();
        file.write_all(&[1u8, 0, 0, 0]).unwrap();
        file.write_all(&[compression as u8, 0, 0, 0, 0, 0, 0, 0])
            .unwrap();
        file.write_all(&[0u8; 16]).unwrap();
        let mut output: Box<dyn Write> = match compression {
            ObjectCompression::None => Box::new(DynWriteWrapper::new(&mut file)),
            ObjectCompression::LZMA => {
                let compressor = XzEncoder::new(DynWriteWrapper::new(&mut file), 9);
                Box::new(compressor)
            }
        };

        let res = ObjectID::from_sha256_stream(
            &mut DynReadWrapper::new(stream),
            &mut DynWriteWrapper::new(&mut output),
        )?;
        drop(output);
        let pos = file.stream_position().unwrap() - 32;
        file.seek(SeekFrom::Start(16)).unwrap();
        file.write(&(pos as u64).to_le_bytes()).unwrap();
        file.write(&(res.0 as u64).to_le_bytes()).unwrap();

        let path = self.root.join("objects").join(res.1.to_path(5));

        mkdir_p(path.parent().unwrap())?;
        rename(&tmp_file, &path)?;

        Ok(res)
    }
}
