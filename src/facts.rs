#![allow(unused)]

use super::services::sample;

use std::io::{Read, Write};
use std::marker::{PhantomData, Sized};

use rmp::decode::{DecodeStringError, MarkerReadError, ValueReadError};
use rmp::encode::ValueWriteError;
use rmp::Marker;

/**********************************************************************
 * Error type
 **********************************************************************/

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    InvalidStructLength { expected: usize, actual: usize },
    DecodeStringError(),
    ValueWriteError(ValueWriteError),
    ValueReadError(ValueReadError),
    MarkerReadError(MarkerReadError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::DecodeStringError()
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::DecodeStringError()
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Self {
        Error::MarkerReadError(err)
    }
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

impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError<'a>) -> Self {
        Error::DecodeStringError()
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
 * Main trait
 **********************************************************************/

pub trait Factual<TT> {
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error>;

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error>
    where
        Self: Sized;

    fn subread<R: Read, T>(rd: &mut Reader<R>) -> Result<T, Error>
    where
        Self: Sized,
        T: Factual<TT>,
    {
        T::read(rd)
    }
}

pub struct Reader<R>
where
    R: Read,
{
    rd: R,
    buf: Vec<u8>,
    marker: Option<Marker>,
}

impl<R> Reader<R>
where
    R: Read,
{
    pub fn new(rd: R) -> Self {
        Self {
            rd,
            buf: Vec::with_capacity(128),
            marker: None,
        }
    }

    fn fetch_marker(&mut self) -> Result<Marker, Error> {
        let marker = match self.marker.take() {
            Some(marker) => Ok(marker),
            None => Ok(rmp::decode::read_marker(&mut self.rd)?),
        };
        marker
    }

    fn read_slice(&mut self, len: usize) -> Result<&[u8], Error> {
        self.buf.resize(len, 0u8);
        self.rd.read_exact(&mut self.buf[..])?;
        Ok(&self.buf[..])
    }

    fn read_array_len(&mut self) -> Result<usize, Error> {
        let marker = self.fetch_marker()?;
        let len = match marker {
            Marker::FixArray(len) => Ok(len as usize),
            Marker::Array16 => Ok(rmp::decode::read_data_u16(self)? as usize),
            Marker::Array32 => Ok(rmp::decode::read_data_u32(self)? as usize),
            _ => Err(ValueReadError::TypeMismatch(marker).into()),
        };
        len
    }

    fn read_str_len(&mut self) -> Result<usize, Error> {
        let marker = self.fetch_marker()?;
        let len = match marker {
            Marker::FixStr(len) => Ok(len as usize),
            Marker::Str8 => Ok(rmp::decode::read_data_u8(self)? as usize),
            Marker::Str16 => Ok(rmp::decode::read_data_u16(self)? as usize),
            Marker::Str32 => Ok(rmp::decode::read_data_u32(self)? as usize),
            _ => Err(ValueReadError::TypeMismatch(marker).into()),
        };
        len
    }
}

impl<R> Read for Reader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.rd.read(buf)
    }
}

impl<T, TT> Factual<TT> for Option<T>
where
    T: Factual<TT>,
{
    fn write<W: Write>(&self, tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        match self {
            Some(v) => v.write(tt, wr)?,
            None => write_nil(wr)?,
        };

        Ok(())
    }

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        match rmp::decode::read_marker(rd)? {
            Marker::Null => Ok(None),
            marker => {
                rd.marker = Some(marker);
                Ok(Some(T::read(rd)?))
            }
        }
    }
}

impl<'a, TT> Factual<TT> for &'a str {
    fn write<W: Write>(&self, _tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_str(wr, self)?;

        Ok(())
    }

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        unimplemented!()
    }
}

impl<'a, TT> Factual<TT> for String {
    fn write<W: Write>(&self, _tt: &TT, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_str(wr, self)?;

        Ok(())
    }

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        let len = rd.read_str_len()?;
        let bytes = rd.read_slice(len)?;
        let res = std::str::from_utf8(bytes)?.to_string();
        Ok(res)
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

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
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

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        let len = rd.read_array_len()?;

        let mut res = Self::with_capacity(len);
        for i in 0..len {
            res.push(T::read(rd)?);
        }
        Ok(res)
    }
}

/**********************************************************************
 * Manual implementation for sample::Cookie
 **********************************************************************/

impl Factual<TranslationTables> for sample::Cookie {
    fn write<W: Write>(&self, tt: &TranslationTables, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, tt.sample__Cookie.len() as u32)?;

        for slot in &tt.sample__Cookie {
            match slot {
                Some(index) => match index {
                    0 => self.key.write(tt, wr)?,
                    1 => self.value.write(tt, wr)?,
                    2 => self.comment.write(tt, wr)?,
                    _ => unreachable!(),
                },
                None => write_nil(wr)?,
            }
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        let len = rd.read_array_len()?;
        if len != 3 {
            return Err(Error::InvalidStructLength {
                expected: 3,
                actual: len,
            });
        }

        // this must be in order
        use TranslationTables as TT;
        let res = sample::Cookie {
            key: Self::subread(rd)?,
            value: Self::subread(rd)?,
            comment: Self::subread(rd)?,
        };
        Ok(res)
    }
}

/**********************************************************************
 * Manual implementation for sample::Emoji
 **********************************************************************/

impl Factual<TranslationTables> for sample::Emoji {
    fn write<W: Write>(&self, tt: &TranslationTables, wr: &mut W) -> Result<(), Error> {
        use rmp::encode::*;
        write_array_len(wr, tt.sample__Emoji.len() as u32)?;

        for slot in &tt.sample__Emoji {
            match slot {
                Some(index) => match index {
                    0 => self.shortcode.write(tt, wr)?,
                    1 => self.image_url.write(tt, wr)?,
                    _ => unreachable!(),
                },
                None => write_nil(wr)?,
            }
        }

        Ok(())
    }

    fn read<R: Read>(rd: &mut Reader<R>) -> Result<Self, Error> {
        let len = rd.read_array_len()?;
        if len != 2 {
            return Err(Error::InvalidStructLength {
                expected: 2,
                actual: len,
            });
        }

        // this must be in order
        use TranslationTables as TT;
        let res = sample::Emoji {
            shortcode: Self::subread(rd)?,
            image_url: Self::subread(rd)?,
        };
        Ok(res)
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

pub fn write<TT, T, W>(t: &T, tt: &TT, wr: &mut W) -> Result<(), Error>
where
    T: Factual<TT>,
    W: Write,
{
    t.write(tt, wr)
}

#[allow(non_snake_case)]
pub struct TranslationTables {
    pub sample__Cookie: Vec<Option<u32>>,
    pub sample__Emoji: Vec<Option<u32>>,
}
