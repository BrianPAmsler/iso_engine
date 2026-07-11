
use std::sync::{Mutex, RwLock};

use crate::error::{self as errors_module, Error};

use error::union;
use itertools::Itertools;
use serde::Serializer;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct Custom(pub String);

union!(std::io::Error, Custom as SerializeError);

impl serde::ser::Error for SerializeError {
    #[doc = r" Used when a [`Serialize`] implementation encounters any error"]
    #[doc = r" while serializing a type."]
    #[doc = r""]
    #[doc = r" The message should not be capitalized and should not end with a"]
    #[doc = r" period."]
    #[doc = r""]
    #[doc = r" For example, a filesystem [`Path`] may refuse to serialize"]
    #[doc = r" itself if it contains invalid UTF-8 data."]
    #[doc = r""]
    #[doc = r" ```edition2021"]
    #[doc = r" # struct Path;"]
    #[doc = r" #"]
    #[doc = r" # impl Path {"]
    #[doc = r" #     fn to_str(&self) -> Option<&str> {"]
    #[doc = r" #         unimplemented!()"]
    #[doc = r" #     }"]
    #[doc = r" # }"]
    #[doc = r" #"]
    #[doc = r" use serde::ser::{self, Serialize, Serializer};"]
    #[doc = r""]
    #[doc = r" impl Serialize for Path {"]
    #[doc = r"     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>"]
    #[doc = r"     where"]
    #[doc = r"         S: Serializer,"]
    #[doc = r"     {"]
    #[doc = r"         match self.to_str() {"]
    #[doc = r"             Some(s) => serializer.serialize_str(s),"]
    #[doc = r#"             None => Err(ser::Error::custom("path contains invalid UTF-8 characters")),"#]
    #[doc = r"         }"]
    #[doc = r"     }"]
    #[doc = r" }"]
    #[doc = r" ```"]
    #[doc = r""]
    #[doc = r" [`Path`]: std::path::Path"]
    #[doc = r" [`Serialize`]: crate::Serialize"]
    fn custom<T>(msg:T) -> Self where T: std::fmt::Display {
        Self::Custom(Custom(format!("{msg}")))
    }
}

fn indent(lines: &mut [String], size: usize) {
    let indent: String = std::iter::repeat(' ').take(size).collect();
    for line in lines {
        *line = indent.clone() + &line;
    }
}

#[derive(Copy, Clone)]
struct __EasyFormatSerializer {
    indent: usize
}

static TUPLE_FIELDS: RwLock<Vec<&'static str>> = RwLock::new(Vec::new());

fn tuple_field_name(index: usize) -> &'static str {
    let mut len = TUPLE_FIELDS.read().unwrap().len();
    while len <= index {
        let field_name = String::leak(format!("{len}"));
        let mut lock = TUPLE_FIELDS.write().unwrap();
        lock.push(field_name);
        len = lock.len();
    }

    TUPLE_FIELDS.read().unwrap()[index]
}

#[allow(unused, reason = "")]
impl Serializer for __EasyFormatSerializer {
    type Ok = Vec<String>;

    type Error = SerializeError;

    type SerializeSeq = SerializeSeq;

    type SerializeTuple = SerializeSeq;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = SerializeStruct;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;


    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        match v {
            true => Ok(vec!["true".to_string()]),
            false => Ok(vec!["false".to_string()]),
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{v}")])
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("\'{v}\'")])
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("\"{v}\"")])
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut byte_string = String::new();

        for byte in v {
            match char::from_u32(*byte as u32).filter(|char| char.is_ascii() && !char.is_ascii_control()) {
                Some(char) => byte_string.push(char),
                None => byte_string += &format!("\\x{byte:02x}")
            }
        }
        Ok(vec![format!("b\"{byte_string}\"")])
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(vec!["None".to_string()])
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize {
        let mut lines = value.serialize(self)?;

        if lines.len() > 1 {
            indent(&mut lines, self.indent);
            lines.insert(0, "Some(".to_string());
            lines.push(")".to_string());

            Ok(lines)
        } else {
            #[allow(clippy::unwrap_used, reason = "len() == 1")]
            let line = lines.pop().unwrap();
            Ok(vec![format!("Some({line})")])
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(vec!["()".to_string()])
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(vec![format!("{name}\n")])
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(vec![variant.to_string()])
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize {
        let mut lines = value.serialize(self)?;

        if lines.len() > 1 {
            indent(&mut lines, self.indent);

            lines.insert(0, format!("{variant}("));
            lines.push(")".into());

            Ok(lines)
        } else {
            Ok(vec![format!("{variant}({})", lines[0])])
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::new(self, len.is_some_and(|len| len >= 10), "[", "]"))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeSeq::new(self, len >= 10, "(", ")"))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        // let fields = (0..len).map(|index| tuple_field_name(index)).collect_vec();
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct::new(self, name))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

struct SerializeSeq {
    serializer: __EasyFormatSerializer,
    elements: Vec<Vec<String>>,
    begin: &'static str,
    end: &'static str,
    multi_line: bool
}

impl SerializeSeq {
    pub fn new(serializer: __EasyFormatSerializer, multi_line_hint: bool, begin: &'static str, end: &'static str) -> SerializeSeq {
        let lines = Vec::new();
        SerializeSeq { serializer, elements: lines, multi_line: multi_line_hint, begin, end }
    }
}

impl serde::ser::SerializeSeq for SerializeSeq {
    type Ok = <__EasyFormatSerializer as Serializer>::Ok;
    type Error = <__EasyFormatSerializer as Serializer>::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize {
        let lines = value.serialize(self.serializer)?;

        if lines.len() > 1 {
            self.multi_line = true;
        }

        self.elements.push(lines);

        Ok(())
    }

    #[allow(clippy::unwrap_used, reason="checked or impossible")]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut lines = vec![self.begin.into()];
        if self.multi_line {
            for mut element in self.elements {
                indent(&mut element, self.serializer.indent);
                lines.extend(element);
                lines.last_mut().unwrap().push(',');
            }
            lines.push(self.end.into());
        } else {
            let mut iter = self.elements.into_iter().peekable();
            while let Some(mut element) = iter.next() {
                let line = element.pop().unwrap();
                lines[0] += &line;

                if iter.peek().is_some() {
                    lines[0] += ", ";
                }
            }
            lines[0].push_str(self.end)
        }
            
        Ok(lines)
    }
}

impl serde::ser::SerializeTuple for SerializeSeq {
    type Ok = <Self as serde::ser::SerializeSeq>::Ok;
    type Error = <Self as serde::ser::SerializeSeq>::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize {
        <Self as serde::ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as serde::ser::SerializeSeq>::end(self)
    }
}

struct SerializeStruct {
    serializer: __EasyFormatSerializer,
    name: &'static str,
    lines: Vec<String>
}

impl SerializeStruct {
    pub fn new(serializer: __EasyFormatSerializer, name: &'static str) -> SerializeStruct {
        SerializeStruct { serializer, name, lines: Vec::new() }
    }
}

impl serde::ser::SerializeStruct for SerializeStruct {
    type Ok = <__EasyFormatSerializer as Serializer>::Ok;
    type Error = <__EasyFormatSerializer as Serializer>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize {
        let mut value_lines = value.serialize(self.serializer)?;

        if value_lines.len() > 1 {
            self.lines.push(format!("{key}:"));
            indent(&mut value_lines, self.serializer.indent);
            self.lines.extend(value_lines);
        } else {
            self.lines.push(format!("{key}: {}", value_lines[0]));
        }

        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        indent(&mut self.lines, self.serializer.indent);

        let mut lines = Vec::new();
        if !self.name.is_empty() {
            lines.push(format!("<{}>", self.name));
        }
        lines.extend(self.lines);

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde::Serialize;

    use crate::engine::resources::serialization::{text_format::__EasyFormatSerializer};

    #[derive(Serialize)]
    enum TestEnum {
        A,
        B,
        C
    }

    #[derive(derive_serialize::Serialize)]
    struct TestDyn {
        pub a: f32,
        pub b: u32,
        pub c: Vec<bool>
    }

    #[derive(Serialize)]
    struct Inner {
        one: i32,
        two: PathBuf,
        three: bool,
        four: TestEnum
    }

    #[derive(Serialize)]
    struct SerializeTest {
        a: String,
        b: Option<u32>,
        c: Vec<f64>,
        d: Inner,
        e: (f32, bool, Option<usize>)
    }

    #[test]
    fn easy_serialize_test() -> Result<(), Box<dyn std::error::Error>> {
        let test = SerializeTest {
            a: "Hello World!".into(),
            b: Some(4),
            c: vec![0.0, 1.0, 2.5],
            d: Inner {
                one: -6,
                two: "abc/def/ghi.jkl".into(),
                three: false,
                four: TestEnum::B
            },
            e: (1.2, true, None)
        };

        let dyn_test = TestDyn {
            a: 12.5,
            b: 7,
            c: vec![false, true, true, false],
        };

        let dyn_output = <TestDyn as crate::engine::resources::serialization::Serialize>::serialize(&dyn_test);

        let output = test.serialize(__EasyFormatSerializer { indent: 2 })?;
        let output2 = dyn_output.serialize(__EasyFormatSerializer { indent: 2 })?;

        for line in output {
            println!("{line}");
        }

        for line in output2 {
            println!("{line}");
        }

        Ok(())
    }
}