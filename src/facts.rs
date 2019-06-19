use super::services::sample;

use std::io::{Read, Write};
use std::marker::{PhantomData, Sized};

use rmp::decode::ValueReadError;
use rmp::encode::ValueWriteError;

/**********************************************************************
 * Error type
 **********************************************************************/

#[derive(Debug)]
pub enum Error {
    ValueWriteError(ValueWriteError),
    ValueReadError(ValueReadError),
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Self {
        Error::ValueWriteError(err)
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Self {
        Error::ValueReadError(err)
    }
}

impl std::error::Error for Error {}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**********************************************************************
 * Serialization helpers
 **********************************************************************/

fn write_null<W: Write>(wr: &mut W) -> Result<(), Error> {
    wr.write_all(&[rmp::Marker::Null.to_u8()])
        .map_err(ValueWriteError::InvalidDataWrite)?;
    Ok(())
}

/**********************************************************************
 * Main trait
 **********************************************************************/

pub trait Factual<TT> {
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error>;

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error>
    where
        Self: Sized;
}

pub struct OptionOf<T, TT>(pub Option<T>, pub PhantomData<TT>)
where
    T: Factual<TT>;

impl<T, TT> Factual<TT> for OptionOf<T, TT>
where
    T: Factual<TT>,
{
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        match &self.0 {
            Some(v) => v.write(tt, wr)?,
            None => write_null(wr)?,
        };

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

pub struct StringOf<'a>(pub &'a str);

impl<'a, TT> Factual<TT> for StringOf<'a> {
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_str(wr, self.0)?;

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

pub struct ArrayOf<'a, T, TT>(pub &'a [T], pub PhantomData<TT>)
where
    T: Factual<TT>;

impl<'a, T, TT> Factual<TT> for ArrayOf<'a, T, TT>
where
    T: Factual<TT>,
{
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, self.0.len() as u32)?;
        for item in self.0 {
            item.write(tt, wr)?;
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

impl<'a, T, TT> Factual<TT> for &'a [T]
where
    T: Factual<TT>,
{
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, self.len() as u32)?;
        for item in *self {
            item.write(tt, wr)?;
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

impl<'a, T, TT> Factual<TT> for Vec<T>
where
    T: Factual<TT>,
{
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, self.len() as u32)?;
        for item in self {
            item.write(tt, wr)?;
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

/**********************************************************************
 * Helpers
 **********************************************************************/

pub fn array_of<'a, T, TT>(v: &'a [T]) -> ArrayOf<'a, T, TT>
where
    T: Factual<TT>,
{
    ArrayOf(v, PhantomData)
}

/**********************************************************************
 * Manual implementation for sample::Cookie
 **********************************************************************/

impl Factual<TranslationTables> for sample::Cookie {
    fn write<W: Write>(&self, tt: &TranslationTables, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, 3)?;

        for slot in &tt.sample__Cookie {
            match slot {
                Some(index) => match index {
                    0 => StringOf(&self.key).write(tt, wr)?,
                    1 => StringOf(&self.value).write(tt, wr)?,
                    2 => OptionOf(self.comment.as_ref().map(|x| StringOf(&x)), PhantomData)
                        .write(tt, wr)?,
                    _ => panic!(
                        "sample::Cookie::write: don't have field with index {index} to write",
                        index = index
                    ),
                },
                None => write_null(wr)?,
            }
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut R) -> Result<Self, Error> {
        unimplemented!()
    }
}

/*

~~ Case A ~~

Server has:
    Cookie { key, value, comment }

Client has:
    Cookie { key, value }

The client's transformation table is:
    [Some(0), Some(1), None]

The server's transformation table is:
    [Some(0), Some(1)]

*/

/*

~~ Case B ~~

Server has:
    Cookie { value, key }

Client has:
    Cookie { key, value }

The client's transformation table is:
    [Some(1), Some(0)]

The server's transformation table is:
    [Some(1), Some(0)]

*/

/*

~~ Case B ~~

Server has:
    Cookie { key, value }

Client has:
    Cookie { name, value }

The client's transformation table is:
    [None, Some(1)]

The server's transformation table is:
    [None, Some(1)]

*/

pub struct TranslationTables {
    pub sample__Cookie: Vec<Option<u32>>,
}
