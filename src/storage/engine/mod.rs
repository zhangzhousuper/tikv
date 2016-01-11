use self::memory::BTreeEngine;
use std::{error, result};
use std::fmt::{self, Display, Formatter};
use self::rocksdb::RocksEngine;

mod memory;
mod rocksdb;

#[derive(Debug)]
pub enum Modify<'a> {
    Delete(&'a [u8]),
    Put((&'a [u8], &'a [u8])),
}

pub trait Engine {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn seek(&self, key: &[u8]) -> Result<Option<(Vec<u8>, Vec<u8>)>>;
    fn write(&mut self, batch: Vec<Modify>) -> Result<()>;

    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.write(vec![Modify::Put((key, value))])
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.write(vec![Modify::Delete(key)])
    }
}

#[derive(Debug)]
pub enum Descriptor<'a> {
    Memory,
    RocksDBPath(&'a str),
}

pub fn new_engine(desc: Descriptor) -> Result<Box<Engine>> {
    match desc {
        Descriptor::Memory => Ok(Box::new(BTreeEngine::new())),
        Descriptor::RocksDBPath(path) => {
            RocksEngine::new(path).map(|engine| -> Box<Engine> { Box::new(engine) })
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Other(Box<error::Error + Send + Sync>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Other(ref error) => Display::fmt(error, f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Other(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Other(ref e) => e.cause(),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::{Descriptor, Engine, Modify};

    #[test]
    fn memory() {
        let mut e = super::new_engine(Descriptor::Memory).unwrap();
        get_put(&mut *e);
        batch(&mut *e);
    }

    #[test]
    fn rocksdb() {
        let mut e = super::new_engine(Descriptor::RocksDBPath("/tmp/rocks")).unwrap();
        get_put(&mut *e);
        batch(&mut *e);
    }

    fn assert_has(engine: &Engine, key: &[u8], value: &[u8]) {
        assert_eq!(engine.get(key).unwrap().unwrap(), value);
    }

    fn assert_none(engine: &Engine, key: &[u8]) {
        assert_eq!(engine.get(key).unwrap(), None);
    }

    fn get_put(engine: &mut Engine) {
        assert_none(engine, b"x");
        engine.put(b"x", b"1").unwrap();
        assert_has(engine, b"x", b"1");
        engine.put(b"x", b"2").unwrap();
        assert_has(engine, b"x", b"2");
        engine.delete(b"x").unwrap();
        assert_none(engine, b"x");
    }

    fn batch(engine: &mut Engine) {
        engine.write(vec![Modify::Put((b"x", b"1")), Modify::Put((b"y", b"2"))]).unwrap();
        assert_has(engine, b"x", b"1");
        assert_has(engine, b"y", b"2");

        engine.write(vec![Modify::Delete(b"x"), Modify::Delete(b"y")]).unwrap();
        assert_none(engine, b"y");
        assert_none(engine, b"y");
    }
}
