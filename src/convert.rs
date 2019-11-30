use chrono::{DateTime, Utc};
use prost_types;
use prost_types::{Any, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::result::{self, Result};
use crate::{S2ProtoPack, S2ProtoUnpack};

macro_rules! impl_option {
  ($rust:ty => $proto:ty) => {
    impl S2ProtoPack<Option<$proto>> for Option<$rust> {
      fn pack(self) -> Result<Option<$proto>> {
        if let Some(value) = self {
          Ok(Some(value.pack()?))
        } else {
          Ok(None)
        }
      }
    }

    impl S2ProtoUnpack<Option<$proto>> for Option<$rust> {
      fn unpack(value: Option<$proto>) -> Result<Option<$rust>> {
        if let Some(value) = value {
          Ok(Some(<$rust>::unpack(value)?))
        } else {
          Ok(None)
        }
      }
    }

    impl S2ProtoPack<Option<$proto>> for $rust {
      fn pack(self) -> Result<Option<$proto>> {
        Ok(Some(self.pack()?))
      }
    }

    impl S2ProtoUnpack<Option<$proto>> for $rust {
      fn unpack(value: Option<$proto>) -> Result<$rust> {
        if let Some(value) = value {
          Ok(<$rust>::unpack(value)?)
        } else {
          Err(result::Error::ValueNotPresent)
        }
      }
    }
  };
}

// JSON value

/// Proto Buffers: Schemas other than http, https (or the empty schema) might be
/// used with implementation specific semantics.
const JSON_TYPE_URL: &str = "s2/json";

impl S2ProtoPack<Any> for Value {
  fn pack(self) -> Result<Any> {
    Ok(Any {
      type_url: JSON_TYPE_URL.to_string(),
      value: serde_json::to_vec(&self).context(result::Json)?,
    })
  }
}

impl S2ProtoUnpack<Any> for Value {
  fn unpack(Any { type_url, value }: Any) -> Result<Value> {
    if type_url == JSON_TYPE_URL {
      serde_json::from_reader(&value as &[u8]).context(result::Json)
    } else {
      Err(result::Error::JsonTypeUrlUnknown { type_url })
    }
  }
}

impl_option!(Value => Any);

/// Helper type to convert any serializable type from/to `google.protobuf.Any`
pub struct Json<T>(pub T);

impl<T> S2ProtoPack<Any> for Json<T>
where
  T: Serialize + for<'de> Deserialize<'de>,
{
  fn pack(self) -> Result<Any> {
    pack_any(self.0)
  }
}

impl<T> S2ProtoUnpack<Any> for Json<T>
where
  T: Serialize + for<'de> Deserialize<'de>,
{
  fn unpack(value: Any) -> Result<Json<T>> {
    unpack_any(value).map(Json)
  }
}

impl<T> S2ProtoPack<Option<Any>> for Option<Json<T>>
where
  T: Serialize + for<'de> Deserialize<'de>,
{
  fn pack(self) -> Result<Option<Any>> {
    if let Some(value) = self {
      Ok(Some(value.pack()?))
    } else {
      Ok(None)
    }
  }
}

impl<T> S2ProtoUnpack<Option<Any>> for Option<Json<T>>
where
  T: Serialize + for<'de> Deserialize<'de>,
{
  fn unpack(value: Option<Any>) -> Result<Option<Json<T>>> {
    if let Some(value) = value {
      Ok(Some(Json::<T>::unpack(value)?))
    } else {
      Ok(None)
    }
  }
}

pub fn pack_any<T>(value: T) -> Result<Any>
where
  T: Serialize,
{
  serde_json::to_value(&value).context(result::Json)?.pack()
}

pub fn unpack_any<T>(value: Any) -> Result<T>
where
  T: for<'de> Deserialize<'de>,
{
  let value = Value::unpack(value)?;
  Ok(serde_json::from_value(value).context(result::Json)?)
}

// Timestamp

impl S2ProtoPack<Timestamp> for DateTime<Utc> {
  fn pack(self) -> Result<Timestamp> {
    Ok(Timestamp {
      seconds: self.timestamp(),
      nanos: self.timestamp_subsec_nanos() as i32,
    })
  }
}

impl S2ProtoUnpack<Timestamp> for DateTime<Utc> {
  fn unpack(Timestamp { seconds, nanos }: Timestamp) -> Result<DateTime<Utc>> {
    let dt = chrono::NaiveDateTime::from_timestamp(seconds, nanos as u32);
    Ok(DateTime::from_utc(dt, Utc))
  }
}

impl_option!(DateTime<Utc> => Timestamp);

// Wrappers

macro_rules! impl_self {
  (
    $($ty:ty),*
  ) => {
    $(
      impl S2ProtoPack<$ty> for $ty {
        fn pack(self) -> Result<$ty> {
          Ok(self)
        }
      }

      impl S2ProtoUnpack<$ty> for $ty {
        fn unpack(value: $ty) -> Result<$ty> {
          Ok(value)
        }
      }
    )*
  }
}

impl_self! {
  f32, Option<f32>,
  f64, Option<f64>,
  i64, Option<i64>,
  u64, Option<u64>,
  i32, Option<i32>,
  u32, Option<u32>,
  bool, Option<bool>,
  String, Option<String>
}
