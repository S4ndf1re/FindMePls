use std::{borrow::Cow, marker::PhantomData, path::PathBuf};

use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::Result;

pub trait Storeable {
    fn filename<'a>(&'a self) -> Result<Cow<'a, str>>;
    fn as_bytes<'a>(&'a self) -> Result<Cow<'a, Vec<u8>>>;
    fn change_from_bytes(&mut self, bytes: &[u8]);
}

#[derive(Debug)]
pub struct FileStorage<D> {
    path: PathBuf,
    d_data: PhantomData<D>,
}

impl<D> FileStorage<D>
where
    D: Storeable,
{
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            d_data: PhantomData {},
        }
    }

    pub async fn store(&self, data: &D) -> Result<()> {
        let mut path = self.path.clone();
        dbg!(&path);
        create_dir_all(path.clone()).await?;
        dbg!(&path);
        path.push(data.filename()?.as_ref());
        let mut file = File::create(path).await?;
        file.write_all(&(data.as_bytes()?)).await?;

        Ok(())
    }

    pub async fn read(&self, data: &mut D) -> Result<()> {
        let mut path = self.path.clone();
        path.push(data.filename()?.as_ref());
        dbg!(&path);
        let mut file = File::open(path).await?;

        let mut vec = vec![];
        let _ = file.read_to_end(&mut vec).await?;

        data.change_from_bytes(&vec);
        Ok(())
    }
}
