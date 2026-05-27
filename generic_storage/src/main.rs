use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::{error::Error, marker::PhantomData};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

// Trait with generic
pub trait Serializer {
    fn to_bytes<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: BorshSerialize + SerdeSerialize + SchemaWrite<DefaultConfig, Src = T> + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>;

    fn from_bytes<T>(&self, data: &[u8]) -> Result<T, Box<dyn Error>>
    where
        T: BorshDeserialize + for<'a> SerdeDeserialize<'a> + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>;
}
pub struct Borsh;

pub struct Wincode;
pub struct Json;

// implementations of Serializer for each serializer struct
impl Serializer for Borsh {
    fn to_bytes<T: BorshSerialize>(&self, data: &T) -> Result<Vec<u8>,Box<dyn Error>> 
    where
        T: BorshSerialize + SerdeSerialize
    
    {
        let bytes = borsh::to_vec(data)?;
        Ok(bytes)
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, Box<dyn Error>>
    where
        T: BorshDeserialize + for <'a>SerdeDeserialize<'a>
    {
        let deserialized = borsh::from_slice(bytes)?;
        Ok(deserialized)
    }
}


impl Serializer for Wincode {
    fn to_bytes<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: SerdeSerialize + BorshSerialize + SchemaWrite<DefaultConfig, Src = T>,
    {
        Ok(wincode::serialize(data)?)
    }

    fn from_bytes<T>(&self, data: &[u8]) -> Result<T, Box<dyn Error>>
    where
        T: for<'a> SerdeDeserialize<'a> + BorshDeserialize + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>,
    {
        Ok(wincode::deserialize(data)?)
    }
}

impl Serializer for Json {
    fn to_bytes<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: BorshSerialize + SerdeSerialize,
    {
        let bytes = serde_json::to_vec(data)?;
        Ok(bytes)
    }

    fn from_bytes<T>(&self, data: &[u8]) -> Result<T, Box<dyn Error>>
    where
        T: BorshDeserialize + for<'a> SerdeDeserialize<'a>,
    {
        let deserialized = serde_json::from_slice(data)?;
        Ok(deserialized)
    }
}


// Storage struct and Implementation
pub struct Storage<T, S> {
    serializer: S,
    data: Option<Vec<u8>>,
    _phantom: PhantomData<T>
}


impl<T, S: Serializer> Storage<T, S>
where
    T: BorshSerialize + BorshDeserialize
        + SerdeSerialize + for<'a> SerdeDeserialize<'a>
        + SchemaWrite<DefaultConfig, Src = T>
        + for<'a> SchemaRead<'a, DefaultConfig, Dst = T>,
{
    pub fn new(serializer: S) -> Self {
        Self {
            serializer,
            data: None,
            _phantom: PhantomData
        }
    }

    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn Error>> {
        self.data = Some(self.serializer.to_bytes(value)?);
        Ok(())
    }

   pub fn load(&self) -> Result<T, Box<dyn Error>> {
        let bytes = self.data.as_ref().ok_or("no data stored")?;
        self.serializer.from_bytes(bytes)
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}

fn main() {
    println!("Hello, world!");
}

#[derive(BorshSerialize, BorshDeserialize, SerdeSerialize, SerdeDeserialize, SchemaRead, SchemaWrite, Debug, PartialEq)]
pub struct Person {
    name: String,
    is_active: bool,
    age: u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_person() -> Person {
        Person { 
            name: "Turbin3 Sol".to_string(),
            is_active: true,
            age: 30
        }
    }

    #[test]
    fn test_borsh() {
        let person = new_person();
        let mut storage: Storage<Person, Borsh> = Storage::new(Borsh);
        storage.save(&person).unwrap();
        assert!(storage.has_data());
        let loaded = storage.load().unwrap();
        assert_eq!(loaded, person);
    }

    #[test]
    fn test_wincode() {
        let person = new_person();
        let mut storage: Storage<Person, Wincode> = Storage::new(Wincode);
        storage.save(&person).unwrap();
        assert!(storage.has_data());
        let loaded = storage.load().unwrap();
        assert_eq!(loaded, person)
    }

    #[test]
    fn test_serd_json(){
        let person = new_person();
        let mut storage: Storage<Person, Json> = Storage::new(Json);
        storage.save(&person).unwrap();
        assert!(storage.has_data());
        let loaded = storage.load().unwrap();
        assert_eq!(loaded, person)
    }
}
