use std::{any::Any, borrow::Cow, cell::{LazyCell, RefCell}, collections::HashMap, fmt::Debug, marker::PhantomData, path::PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum FieldValue {
    Int(i128),
    UInt(u128),
    Bool(bool),
    Float(f64),
    String(String),
    Path(PathBuf),
    List(Vec<FieldValue>),
    Struct(StructRepr),
    Option(Option<Box<FieldValue>>),
    Enum {
        variant: String,
        value: Option<Box<FieldValue>>
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StructRepr {
    type_name: Cow<'static, str>,
    fields: HashMap<Cow<'static, str>, FieldValue>
}

impl StructRepr {
    pub fn new(type_name: &'static str, fields: HashMap<&'static str, FieldValue>) -> StructRepr {
        let type_name = type_name.into();
        let fields = fields.into_iter().map(|(k, v)| (k.into(), v)).collect();

        StructRepr { type_name, fields }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&mut self) -> &mut HashMap<Cow<'static, str>, FieldValue> {
        &mut self.fields
    }
}

pub trait Serialize: Any {
    fn serialize(&self) -> StructRepr;
    fn deserialize(fields: StructRepr) -> Result<Box<Self>, DeserializeError> where Self: Sized;

    fn type_name_val(&self) -> &'static str;
    fn type_name() -> &'static str where Self: Sized;
}

pub trait AsSerialize {
    fn as_serialize(&self) -> Option<&dyn Serialize> {
        None
    }
}

#[doc(hidden)]
pub mod hidden {
    use std::{collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque}, marker::PhantomData, ops::Deref, path::PathBuf, sync::{Arc, Mutex}};

    use derive_serialize::tuple_impl;
    use itertools::Itertools;

    use crate::{engine::{game_object::component::Component, resources::serialization::{CompSer, Deserializer, DeserializerEnum, FieldValue, Serialize, TYPE_DICT, error::{DeserializeError, IncorrectFields, IncorrectType}}}};

    macro_rules! primitive_impl {
        ($variant:ident: $($type:ty),* as $as:ident) => {
            $(
            impl ConvertFrom<$type> for FieldValue {
                fn convert_from(value: &$type) -> FieldValue {
                    FieldValue::$variant(*value as $as)
                }
            }

            impl TryConvertFrom<FieldValue> for $type {
                fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
                    match value {
                        FieldValue::$variant(value) => Ok(value as $type),
                        _ => Err(IncorrectFields)?
                    }
                }
            }
            )*
        };
        ($variant:ident: $($type:ty),*) => {
            $(
            impl ConvertFrom<$type> for FieldValue {
                fn convert_from(value: &$type) -> FieldValue {
                    FieldValue::$variant(value.clone())
                }
            }

            impl TryConvertFrom<FieldValue> for $type {
                fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
                    match value {
                        FieldValue::$variant(value) => Ok(value),
                        _ => Err(IncorrectFields)?
                    }
                }
            }
            )*
        };
    }

    primitive_impl!(Int: i128, i64, i32, i16, i8 as i128);
    primitive_impl!(UInt: u128, u64, u32, u16, u8 as u128);
    primitive_impl!(Float: f64, f32 as f64);
    primitive_impl!(String: String);
    primitive_impl!(Path: PathBuf);
    primitive_impl!(Bool: bool);

    tuple_impl!(10);
    
    pub trait TryConvertFrom<T> where Self: Sized {
        fn try_convert_from(value: T) -> Result<Self, DeserializeError>;
    }

    pub trait TryConvertInto<T> where Self: Sized {
        fn try_convert_into(self) -> Result<T, DeserializeError>;
    }

    impl<A, B: TryConvertFrom<A>> TryConvertInto<B> for A {
        fn try_convert_into(self) -> Result<B, DeserializeError> {
            TryConvertFrom::try_convert_from(self)
        }
    }

    pub trait ConvertFrom<T> where Self: Sized {
        fn convert_from(value: &T) -> Self;
    }

    pub trait ConvertInto<T> where Self: Sized {
        fn convert_into(&self) -> T;
    }

    impl<A, B: ConvertFrom<A>> ConvertInto<B> for A {
        fn convert_into(&self) -> B {
            ConvertFrom::convert_from(self)
        }
    }

    impl<T: Serialize> ConvertFrom<T> for FieldValue {
        fn convert_from(value: &T) -> Self {
            FieldValue::Struct(value.serialize())
        }
    }

    impl<T: Serialize> TryConvertFrom<FieldValue> for T {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            match value {
                FieldValue::Struct(fields) => Ok(*T::deserialize(fields)?),
                _ => Err(IncorrectFields)?
            }
        }
    }

    impl<T> ConvertFrom<Option<T>> for FieldValue
    where
        FieldValue: ConvertFrom<T>
    {
        fn convert_from(value: &Option<T>) -> Self {
            FieldValue::Option(value.as_ref().map(ConvertFrom::<T>::convert_from).map(Box::new))
        }
    }
    
    impl<T> TryConvertFrom<FieldValue> for Option<T>
    where
        T: TryConvertFrom<FieldValue>
    {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            match value {
                FieldValue::Option(option) => option.map(|boxed| *boxed).map(TryConvertFrom::try_convert_from).transpose(),
                _ => Err(IncorrectFields)?
            }
        }
    }

    impl<T> ConvertFrom<Arc<T>> for FieldValue
    where
        FieldValue: ConvertFrom<T>
    {
        fn convert_from(value: &Arc<T>) -> Self {
            value.deref().convert_into()
        }
    }
    
    impl<T> TryConvertFrom<FieldValue> for Arc<T>
    where
        T: TryConvertFrom<FieldValue>
    {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            Ok(Arc::new(value.try_convert_into()?))
        }
    }

    impl<T> ConvertFrom<Mutex<T>> for FieldValue
    where
        FieldValue: ConvertFrom<T>
    {
        fn convert_from(value: &Mutex<T>) -> Self {
            #[allow(clippy::unwrap_used, reason="Poisoned lock should panic.")]
            value.lock().unwrap().deref().convert_into()
        }
    }
    
    impl<T> TryConvertFrom<FieldValue> for Mutex<T>
    where
        T: TryConvertFrom<FieldValue>
    {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            Ok(Mutex::new(value.try_convert_into()?))
        }
    }

    impl<T, const N: usize> ConvertFrom<[T; N]> for FieldValue
    where
        FieldValue: ConvertFrom<T>
    {
        fn convert_from(value: &[T; N]) -> Self {
            FieldValue::List(value.iter().map(|element| element.convert_into()).collect())
        }
    }

    impl<T, const N: usize> TryConvertFrom<FieldValue> for [T; N]
    where
        T: TryConvertFrom<FieldValue>
    {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            match value {
                FieldValue::List(list) => {
                    let list: Vec<T> = list.into_iter().map(|field| field.try_convert_into()).try_collect()?;
                    
                    Ok(list.try_into().map_err(|_| IncorrectFields)?)
                },
                _ => Err(IncorrectFields)?
            }
        }
    }

    impl ConvertFrom<Vec<FieldValue>> for FieldValue {
        fn convert_from(value: &Vec<FieldValue>) -> Self {
            FieldValue::List(value.clone())
        }
    }

    impl TryConvertFrom<FieldValue> for Vec<FieldValue> {
        fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
            match value {
                FieldValue::List(vec) => Ok(vec),
                _ => Err(IncorrectType)?
            }
        }
    }

    macro_rules! iter_impl_try_from {
        ($type:ident <$($generic:ident),*> $(where $($tokens:tt)*)?) => {
            impl<$($generic),*> TryConvertFrom<FieldValue> for $type<$($generic),*>
            where
                $($generic: TryConvertFrom<FieldValue>),*,
                $($($tokens)*)*
            {
                fn try_convert_from(value: FieldValue) -> Result<Self, DeserializeError> {
                    match value {
                        FieldValue::List(list) => {
                            list.into_iter().map(|field| field.try_convert_into()).try_collect()
                        },
                        _ => Err(IncorrectFields)?
                    }
                }
            }
        };
    }

    macro_rules! iter_impl {
        ($type:ident <$($generic:ident),*> $(where $($tokens:tt)*)?) => {
            impl<$($generic),*> ConvertFrom<$type<$($generic),*>> for FieldValue
            where
                $(FieldValue: ConvertFrom<$generic>),*,
                $($($tokens)*)*
            {
                fn convert_from(value: &$type<$($generic),*>) -> Self {
                    FieldValue::List(value.iter().map(|element| element.convert_into()).collect())
                }
            }

            iter_impl_try_from!($type <$($generic),*> $(where $($tokens)*)?);
        };
        (map $type:ident <$($generic:ident),*> $(where $($tokens:tt)*)?) => {
            impl<$($generic),*> ConvertFrom<$type<$($generic),*>> for FieldValue
            where
                $(FieldValue: ConvertFrom<$generic>),*,
                $($($tokens)*)*
            {
                fn convert_from(value: &$type<$($generic),*>) -> Self {
                    FieldValue::List(
                        value.iter().map(|element| FieldValue::List(vec![element.0.convert_into(), element.1.convert_into()])).collect(),
                    )
                }
            }

            iter_impl_try_from!($type <$($generic),*> $(where $($tokens)*)?);
        };
    }

    iter_impl!(Vec<T>);
    iter_impl!(VecDeque<T>);
    iter_impl!(LinkedList<T>);

    iter_impl!(map HashMap<K, V> where K: Eq + std::hash::Hash);
    iter_impl!(map BTreeMap<K, V> where K: Ord);

    iter_impl!(HashSet<T> where T: Eq + std::hash::Hash);
    iter_impl!(BTreeSet<T> where T: Ord);

    iter_impl!(BinaryHeap<T> where T: Ord);

    pub struct Registration(pub(in crate::engine::resources::serialization) DeserializerEnum);

    pub trait Register {
        fn register() -> Registration where Self: Sized;
    }

    impl<T: Serialize> Register for Wrapper<T> {
        fn register() -> Registration {
            Registration(DeserializerEnum::NonComponent(Deserializer::<dyn Serialize + 'static>::new::<T>()))
        }
    }

    pub struct Wrapper<T: Serialize>(PhantomData<T>);

    impl<T: Serialize + Component> Wrapper<T> {
        pub fn register() -> Registration {
            Registration(DeserializerEnum::Component(Deserializer::<dyn CompSer + 'static>::new::<T>()))
        }
    }

    pub fn __register_serializable_types(components: impl IntoIterator<Item = (&'static str, Registration)>) {
        TYPE_DICT.with(|type_dict| {
            let mut type_dict = type_dict.borrow_mut();
            for (k, v) in components.into_iter() {
                // Name conflicts are probably impossible, but if it ever happens I want to know about it.
                // Well, this could certainly happen if someone accidentally called register_serializable_types() twice.
                if type_dict.contains_key(k) {
                    panic!("Serialization name conflict for name: {k}");
                }

                type_dict.insert(k, v.0);
            }
        })
    }
}

pub trait CompSer: Component + Serialize {}
impl<T: Component + Serialize> CompSer for T {}

enum DeserializerEnum {
    Component(Deserializer<dyn CompSer>),
    NonComponent(Deserializer<dyn Serialize>)
}

impl Debug for DeserializerEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializerEnum::Component(_) => write!(f, "Component"),
            DeserializerEnum::NonComponent(_) => write!(f, "NonComponent"),
        }
    }
}

trait DeserializeFn<S: Serialize + ?Sized>: Fn(StructRepr) -> Result<Box<S>, DeserializeError> {}
impl<S: Serialize + ?Sized, T: Fn(StructRepr) -> Result<Box<S>, DeserializeError>> DeserializeFn<S> for T {}

struct Deserializer<T: ?Sized + Serialize> {
    deserialize: Box<dyn DeserializeFn<T>>,
    _pd: PhantomData<T>
}

impl Deserializer<dyn Serialize> {
    fn new<T: Serialize + Sized + 'static>() -> Deserializer<dyn Serialize> {
        Deserializer { deserialize: Box::new(|fields| Ok(T::deserialize(fields)?)), _pd: PhantomData }
    }

    fn deserialize(&self, fields: StructRepr) -> Result<Box<dyn Serialize>, DeserializeError> {
        (self.deserialize)(fields)
    }
}

impl Deserializer<dyn CompSer> {
    fn new<T: Serialize + Component + Sized + 'static>() -> Deserializer<dyn CompSer> {
        Deserializer { deserialize: Box::new(|fields| Ok(T::deserialize(fields)?)), _pd: PhantomData }
    }

    fn deserialize(&self, fields: StructRepr) -> Result<Box<dyn CompSer>, DeserializeError> {
        (self.deserialize)(fields)
    }
}

thread_local! {
    static TYPE_DICT: LazyCell<RefCell<HashMap<&'static str, DeserializerEnum>>> = LazyCell::new(|| RefCell::new(HashMap::new()));
}

pub enum DeserializedType {
    Component(Box<dyn CompSer>),
    NonComponent(Box<dyn Serialize>)
}

pub fn dyn_deserialize(fields: StructRepr) -> Result<DeserializedType, DeserializeError> {
    TYPE_DICT.with(|type_dict| {
        let type_dict = type_dict.borrow();
        let deserializer = type_dict.get(fields.type_name()).ok_or(UnknownType(fields.type_name().to_string()))?;

        Ok(match deserializer {
            DeserializerEnum::Component(deserializer) => DeserializedType::Component(deserializer.deserialize(fields)?),
            DeserializerEnum::NonComponent(deserializer) => DeserializedType::NonComponent(deserializer.deserialize(fields)?),
        })
    })
}

#[macro_export]
macro_rules! register_serializable_types {
    ($($type:ty),*) => {{
        use $crate::engine::resources::serialization::Serialize;
        use $crate::engine::resources::serialization::hidden::Wrapper;
        #[allow(unused, reason="The macro cannot tell if this import is necessary or not.")]
        use $crate::engine::resources::serialization::hidden::Register;

        $crate::engine::resources::serialization::hidden::__register_serializable_types([$(
            // This is absolutely disgusting, but I can't think of another solution.
            // Wrapper<T> only implements register() when T implements both Serialize and Component,
            // but the Register trait provides register() for anything that implements Serialize.
            // Type impementations have priority over trait implementations, so the wrapper's
            // function gets called when it exists. Maybe this was intended to be used this way,
            // but it feels wrong to me.
            (<$type>::type_name(), <Wrapper<$type>>::register())
        ),*]);
    }};
}

pub use register_serializable_types;

use derive_serialize::Serialize;

use crate::engine::{game_object::component::Component, resources::serialization::error::{DeserializeError, UnknownType}};

pub mod error {
    use error::{Error, union};
    use crate::error as errors_module;

    #[derive(Error, Debug)]
    #[error("Deserialized struct has incorrect fields.")]
    pub struct IncorrectFields;

    #[derive(Error, Debug)]
    #[error("Deserialized value has incorrect type.")]
    pub struct IncorrectType;

    #[derive(Error, Debug)]
    #[error("Unknown type: {0}")]
    pub struct UnknownType(pub String);

    #[derive(Error, Debug)]
    #[error("Type cannot be constructed.")]
    pub struct NoConstructor;

    union!(IncorrectFields, UnknownType, IncorrectType as DeserializeError);
}


#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::io::Read;

    use crate::engine::{game_object::component::Component, resources::serialization::{CompSer, DeserializedType, TYPE_DICT, dyn_deserialize}};

    #[derive(Debug, derive_serialize::Serialize)]
    #[allow(unused, reason="test")]
    struct TestStruct {
        pub name: String,
        pub a: u32,
        pub b: f32,
        pub c: i32,
        pub d: u64,
        #[serialized]
        e: f64,
        #[non_serialized]
        pub f: i64,
        g: u128
    }

    #[derive(Debug, derive_serialize::Serialize)]
    #[allow(unused, reason="test")]
    struct TestStruct2 {
        pub name: String,
        pub a: u32,
        pub b: f32,
        pub c: i32,
        pub d: u64,
        #[serialized]
        e: f64,
        #[non_serialized]
        pub f: i64,
        g: u128
    }

    impl Component for TestStruct {
        fn init(&mut self, _engine: &mut crate::engine::Engine, _owner: crate::engine::game_object::ObjectID) -> crate::error::any::Result<()> {
            println!("{:?}", self);
            Ok(())
        }
    }

    #[test]
    fn dyn_test() -> Result<(), crate::error::any::Error> {
        
        #[allow(unsafe_code, reason="These values never actually get used, so for the sake of the test it's fine. Creating a valid Engine instance in a test is more trouble than it's worth.")]
        let mut engine = unsafe { std::mem::transmute::<[u8; std::mem::size_of::<crate::engine::Engine>()], crate::engine::Engine>([0u8; _]) };
        #[allow(unsafe_code, reason="See above.")]
        let owner = unsafe { std::mem::zeroed() };

        register_serializable_types!(TestStruct, TestStruct2);

        // TODO: Assert instead of print
        TYPE_DICT.with(|type_dict| println!("{:?}", type_dict.borrow().deref()));

        let mut test_struct: Box<dyn CompSer> = Box::new(TestStruct {
            name: "Test".to_owned(),
            a: 1,
            b: 2.0,
            c: 3,
            d: 4,
            e: 5.0,
            f: 6,
            g: 7
        });

        let mut stdout = shh::stdout()?;
        test_struct.init(&mut engine, owner)?;
        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        assert_eq!(&output, "TestStruct { name: \"Test\", a: 1, b: 2.0, c: 3, d: 4, e: 5.0, f: 6, g: 7 }\n");

        let serialized = test_struct.serialize();
        let DeserializedType::Component(mut deserialized) = dyn_deserialize(serialized)? else { panic!("Not component") };

        deserialized.init(&mut engine, owner)?;
        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        assert_eq!(&output, "TestStruct { name: \"Test\", a: 1, b: 2.0, c: 3, d: 4, e: 5.0, f: 0, g: 0 }\n");

        // Engine is not valid, so stop destructor to prevent UB
        std::mem::forget(engine);

        Ok(())
    }
}