use std::future::Future;
use std::time;
use indexmap::IndexMap;
use anyhow::Result;
use async_recursion::async_recursion;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::error::Amf0ReadError;
use crate::error::Amf0ReadError::{CircularReference, InvalidDate, OutOfRangeReference, UnknownMarker, Unsupported};

mod marker {
    pub const NUMBER: u8 = 0x00;
    pub const BOOLEAN: u8 = 0x01;
    pub const STRING: u8 = 0x02;
    pub const OBJECT: u8 = 0x03;
    pub const MOVIECLIP: u8 = 0x04;
    // reserved, not supported
    pub const NULL: u8 = 0x05;
    pub const UNDEFINED: u8 = 0x06;
    pub const REFERENCE: u8 = 0x07;
    pub const ECMA_ARRAY: u8 = 0x08;
    pub const OBJECT_END_MARKER: u8 = 0x09;
    pub const STRICT_ARRAY: u8 = 0x0A;
    pub const DATE: u8 = 0x0B;
    pub const LONG_STRING: u8 = 0x0C;
    pub const UNSUPPORTED: u8 = 0x0D;
    pub const RECORDSET: u8 = 0x0E;
    // reserved, not supported
    pub const XML_DOCUMENT: u8 = 0x0F;
    pub const TYPED_OBJECT: u8 = 0x10;
    pub const AVMPLUS_OBJECT: u8 = 0x11;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),

    Boolean(bool),

    String(String),

    Object {
        class_name: Option<String>,
        entries: IndexMap<String, Value>,
    },

    Null,

    Undefined,

    EcmaArray {
        entries: IndexMap<String, Value>,
    },

    Array {
        entries: Vec<Value>,
    },

    Date {
        unix_time: time::Duration,
        time_zone: i16,
    },

    XmlDocument(String),
}

impl Value {
    /// Reads an AMF0 encoded `Value` from `reader`.
    ///
    /// Note that reference objects are copied in the decoding phase
    /// for the sake of simplicity of the resulting value representation.
    /// And circular reference are unsupported (i.e., those are treated as errors).
    pub async fn read_from<R>(reader: R) -> Result<Self>
        where
            R: AsyncRead + AsyncReadExt + Unpin + Send,
    {
        Decoder::new(reader).decode().await
    }

    /// Writes the AMF0 encoded bytes of this value to `writer`.
    // pub async fn write_to<W>(&self, writer: W) -> Result<()>
    //     where
    //         W: AsyncWrite + AsyncWriteExt + Unpin + Send,
    // {
    //     Encoder::new(writer).encode(self)
    // }

    /// Tries to convert the value as a `str` reference.
    pub async fn try_as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref x) => Some(x.as_ref()),
            Value::XmlDocument(ref x) => Some(x.as_ref()),
            _ => None,
        }
    }

    /// Tries to convert the value as a `f64`.
    pub fn try_as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(x) => Some(x),
            _ => None,
        }
    }

    /// Tries to convert the value as an iterator of the contained values.
    pub fn try_into_values(self) -> Result<Box<dyn Iterator<Item=Value>>, Self> {
        match self {
            Value::Array { entries } => Ok(Box::new(entries.into_iter().map(|x| x))),
            _ => Err(self),
        }
    }

    /// Tries to convert the value as an iterator of the contained pairs.
    pub fn try_into_pairs(self) -> Result<Box<dyn Iterator<Item=(String, Value)>>, Self> {
        match self {
            Value::EcmaArray { entries } => Ok(Box::new(
                entries
                    .into_iter()
                    .map(|p| (p.0, p.1)),
            )),
            Value::Object { entries, .. } => Ok(Box::new(
                entries
                    .into_iter()
                    .map(|p| (p.0, p.1)),
            )),
            _ => Err(self),
        }
    }
}

/// Makes a `String` value.
pub fn string<T>(t: T) -> Value
    where
        String: From<T>,
{
    Value::String(From::from(t))
}

/// Makes a `Number` value.
pub fn number<T>(t: T) -> Value
    where
        f64: From<T>,
{
    Value::Number(From::from(t))
}

/// Makes an anonymous `Object` value.
pub fn object<I, K>(entries: I) -> Value
    where
        I: Iterator<Item=(K, Value)>,
        String: From<K>,
{
    let mut res: IndexMap<String, Value> = IndexMap::new();
    for (k, v) in entries {
        res.insert(String::from(k), v);
    }
    Value::Object {
        class_name: None,
        entries: res,
    }
}

/// Make a strict `Array` value.
pub fn array(entries: Vec<Value>) -> Value {
    Value::Array { entries }
}

#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
    complexes: Vec<Value>,
}

impl<R> Decoder<R> {
    /// Unwraps this `Decoder`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Get the reference to the underlying reader.
    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// Get the mutable reference to the underlying reader.
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R> Decoder<R>
    where
        R: AsyncRead + AsyncReadExt + Send + Unpin,
{
    /// Makes a new instance.
    pub fn new(inner: R) -> Self {
        Decoder {
            inner,
            complexes: Vec::new(),
        }
    }

    /// Decodes a AMF0 value.
    pub async fn decode(&mut self) -> Result<Value> {
        self.decode_value().await
    }

    /// Clear the reference table of this decoder.
    ///
    /// > Note that object reference indices are local to each message body.
    /// > Serializers and deserializers must reset reference indices to 0 each time a new message is processed.
    /// >
    /// > [AMF 0 Specification: 4.1.3 AMF Message](http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf)
    pub fn clear_reference_table(&mut self) {
        self.complexes.clear();
    }
    #[async_recursion]
    async fn decode_value(&mut self) -> Result<Value> {
        let marker = self.inner.read_u8().await?;
        match marker {
            marker::NUMBER => self.decode_number().await,
            marker::BOOLEAN => self.decode_boolean().await,
            marker::STRING => self.decode_string().await,
            marker::OBJECT => self.decode_object().await,
            marker::MOVIECLIP => Err(Unsupported.into()),
            marker::NULL => Ok(Value::Null),
            marker::UNDEFINED => Ok(Value::Undefined),
            marker::REFERENCE => self.decode_reference().await,
            marker::ECMA_ARRAY => self.decode_ecma_array().await,
            marker::OBJECT_END_MARKER => Err(Unsupported.into()),
            marker::STRICT_ARRAY => self.decode_strict_array().await,
            marker::DATE => self.decode_date().await,
            marker::LONG_STRING => self.decode_long_string().await,
            marker::UNSUPPORTED => Err(Unsupported.into()),
            marker::RECORDSET => Err(Unsupported.into()),
            marker::XML_DOCUMENT => self.decode_xml_document().await,
            marker::TYPED_OBJECT => self.decode_typed_object().await,
            _ => Err(UnknownMarker(marker).into()),
        }
    }
    async fn decode_number(&mut self) -> Result<Value> {
        let n = self.inner.read_f64().await?;
        Ok(Value::Number(n))
    }
    async fn decode_boolean(&mut self) -> Result<Value> {
        let b = self.inner.read_u8().await? != 0;
        Ok(Value::Boolean(b))
    }
    async fn decode_string(&mut self) -> Result<Value> {
        let len = self.inner.read_u16().await? as usize;
        self.read_utf8(len).await.map(Value::String)
    }
    async fn decode_object(&mut self) -> Result<Value> {
        let index = self.complexes.len();
        self.complexes.push(Value::Null);
        let entries = self.decode_pairs().await?;
        let value=Value::Object {
            class_name: None,
            entries,
        };
        self.complexes[index] = value.clone();
        Ok(value)
    }
    async fn decode_reference(&mut self) -> Result<Value> {
        let index = self.inner.read_u16().await? as usize;
        self.complexes
            .get(index)
            .ok_or(OutOfRangeReference(index).into())
            .and_then(|v| {
                if *v == Value::Null {
                    Err(CircularReference.into())
                } else {
                    Ok(v.clone())
                }
            })
    }
    async fn decode_ecma_array(&mut self) -> Result<Value> {
        let index = self.complexes.len();
        self.complexes.push(Value::Null);
        let _count = self.inner.read_u32().await? as usize;
        let entries = self.decode_pairs().await?;
        let value = Value::EcmaArray { entries };
        self.complexes[index] = value.clone();
        Ok(value)
    }
   async  fn decode_strict_array(&mut self) -> Result<Value> {
       let index = self.complexes.len();
       self.complexes.push(Value::Null);
       let count = self.inner.read_u32().await? as usize;
       let mut entries = Vec::new();
       for i in 0..count {
          let _t =  self.decode_value().await?;
           entries.push(_t);
       }

       let value = Value::Array { entries };
       self.complexes[index] = value.clone();
       Ok(value)
    }
    async fn decode_date(&mut self) -> Result<Value> {
        let millis = self.inner.read_f64().await?;
        let time_zone = self.inner.read_i16().await?;
        if !(millis.is_finite() && millis.is_sign_positive()) {
            Err(InvalidDate(millis).into())
        } else {
            Ok(Value::Date {
                unix_time: time::Duration::from_millis(millis as u64),
                time_zone,
            })
        }
    }
   async  fn decode_long_string(&mut self) -> Result<Value> {
        let len = self.inner.read_u32().await? as usize;
        self.read_utf8(len).await.map(Value::String)
    }
    async fn decode_xml_document(&mut self) -> Result<Value> {
        let len = self.inner.read_u32().await? as usize;
        self.read_utf8(len).await.map(Value::XmlDocument)
    }
    async fn decode_typed_object(&mut self) -> Result<Value> {

        let index = self.complexes.len();
        self.complexes.push(Value::Null);
        let len = self.inner.read_u16().await? as usize;
        let class_name = self.read_utf8(len).await?;
        let entries = self.decode_pairs().await?;
        let value =Value::Object {
            class_name: Some(class_name),
            entries,
        };
        self.complexes[index] = value.clone();
        Ok(value)
    }

    async fn read_utf8(&mut self, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        self.inner.read_exact(&mut buf).await?;
        let utf8 = String::from_utf8(buf)?;
        Ok(utf8)
    }
    async fn decode_pairs(&mut self) -> Result<IndexMap<String, Value>> {
        let mut entries = IndexMap::new();
        loop {
            let len = self.inner.read_u16().await? as usize;
            let key = self.read_utf8(len).await?;
            match self.decode_value().await {
                Ok(value) => {
                    entries.insert(key, value);
                }
                Err(e) => {
                    if let Some(Amf0ReadError::UnexpectedObjectEnd) = e.downcast_ref::<Amf0ReadError>() {
                        // 我们已经确定了错误类型，可以按需处理它
                        if key.is_empty() { break; }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(entries)
    }
}