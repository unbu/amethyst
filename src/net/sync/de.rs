

use ecs::{Component, Entities, Entity, SystemData, WriteStorage};

use net::{Error, NetId, NetStat};

use serde::de::{Deserialize, Deserializer, DeserializeSeed, EnumAccess, Error as DeError,
                SeqAccess, VariantAccess, Visitor};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

pub trait ComponentsDeserializer<'a> {
    const COUNT: usize;
    type SystemData: SystemData<'a>;

    fn deserialize_components<'de, A>(&mut self,
                                      entities: &mut Entities<'a>,
                                      stats: &mut WriteStorage<'a, NetStat>,
                                      data: &mut Self::SystemData,
                                      stat: NetStat,
                                      seq: A)
                                      -> Result<Result<(), Error>, A::Error>
        where A: SeqAccess<'de>;

    fn remove_entity(&mut self, entities: &mut Entities<'a>, stat: NetStat) -> Result<(), Error>;
}

pub struct BasicComponentsDeserializer<T> {
    ids: HashMap<NetId, Entity>,
    pd: PhantomData<T>,
}

macro_rules! impl_deserializers {
    ($arity:expr; $($a:ident),*) => {
        impl<'a, $($a,)*> ComponentsDeserializer<'a> for BasicComponentsDeserializer<($($a,)*)>
            where $($a: Component + for<'de> Deserialize<'de>,)*
        {
            type SystemData = (
            $(
                WriteStorage<'a, $a>,
            )*);

            const COUNT: usize = $arity;

            fn remove_entity(&mut self, entities: &mut Entities<'a>, stat: NetStat) -> Result<(), Error> {
                use std::collections::hash_map::Entry::*;

                match self.ids.entry(stat.id()) {
                    Occupied(entry) => {
                        let entity = entry.remove();
                        entities.delete(entity);
                    }
                    _ => {}
                }
                Ok(())
            }

            #[allow(unused_mut, unused_variables)]
            fn deserialize_components<'de, SA>(&mut self,
                                            entities: &mut Entities<'a>,
                                            stats: &mut WriteStorage<'a, NetStat>,
                                            data: &mut Self::SystemData,
                                            stat: NetStat,
                                            mut seq: SA)
                                            -> Result<Result<(), Error>, SA::Error>
                where SA: SeqAccess<'de>
            {
                use std::collections::hash_map::Entry::*;

                #[allow(non_snake_case)]
                let ($(ref mut $a,)*) = *data;
                let entity = match self.ids.entry(stat.id()) {
                    Occupied(entry) => {
                        let entity = *entry.get();
                        let ref mut oldstat = stats.get_mut(entity).expect("Don't touch NetStat!");
                        match oldstat.update(stat) {
                            Err(err) => return Ok(Err(err)),
                            Ok(false) => return Ok(Ok(())),
                            Ok(true) => {}
                        }
                        entity
                    }
                    Vacant(entry) => {
                        let entity = *entry.insert(entities.create());
                        stats.insert(entity, stat);
                        entity
                    }
                };

                let mut index = 0usize;
                $(
                    index += 1;
                    let component = seq.next_element()?
                        .ok_or(
                            DeError::invalid_length(index-1,
                                                    &"Sequence with all syncable components")
                        )?;
                    if let Some(component) = component {
                        $a.insert(entity, component);
                    } else {
                        $a.remove(entity);
                    }
                )*;
                Ok(Ok(()))
            }
        }
    };
}


impl_deserializers!(00;);
impl_deserializers!(01;Z);
//impl_deserializers!(02;Y,Z);
//impl_deserializers!(03;X,Y,Z);
//impl_deserializers!(04;W,X,Y,Z);
//impl_deserializers!(05;V,W,X,Y,Z);
//impl_deserializers!(06;U,V,W,X,Y,Z);
//impl_deserializers!(07;T,U,V,W,X,Y,Z);
//impl_deserializers!(08;S,T,U,V,W,X,Y,Z);
//impl_deserializers!(09;R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(10;Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(11;P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(12;O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(13;N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(14;M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(15;L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(16;K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(17;J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(18;I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(19;H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(20;G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(21;F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(22;E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(23;D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(24;C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(25;B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);
//impl_deserializers!(26;A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z);



pub struct DeserializeEntities<'a, 'b: 'a, T: 'a, S: 'a> {
    deserializer: &'a mut T,
    entities: &'a mut Entities<'b>,
    stats: &'a mut WriteStorage<'b, NetStat>,
    system_data: &'a mut S,
    pd: PhantomData<&'b ()>,
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> DeserializeSeed<'de> for DeserializeEntities<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn deserialize<D>(self, deserializer: D) -> Result<Result<(), Error>, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> Visitor<'de> for DeserializeEntities<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Sequence of `Entity`")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Result<(), Error>, A::Error>
        where A: SeqAccess<'de>
    {
        while let Some(res) = seq.next_element_seed(DeserializeEntity {
            deserializer: self.deserializer,
            entities: self.entities,
            stats: self.stats,
            system_data: self.system_data,
            pd: self.pd,
        })? {
            match res {
                Ok(()) => {},
                Err(err) => return Ok(Err(err)),
            }
        }
        Ok(Ok(()))
    }
}

struct DeserializeEntity<'a, 'b: 'a, T: 'a, S: 'a> {
    deserializer: &'a mut T,
    entities: &'a mut Entities<'b>,
    stats: &'a mut WriteStorage<'b, NetStat>,
    system_data: &'a mut S,
    pd: PhantomData<&'b ()>,
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> DeserializeSeed<'de> for DeserializeEntity<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn deserialize<D>(self, deserializer: D) -> Result<Result<(), Error>, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_tuple_struct("Entity", 2, self)
    }
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> Visitor<'de> for DeserializeEntity<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "NetId followed by components sync-state")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Result<(), Error>, A::Error>
        where A: SeqAccess<'de>
    {
        let stat: NetStat = seq.next_element()?
            .ok_or(DeError::invalid_length(0, &self))?;

        seq.next_element_seed(DeserializeUpdateOrRemove {
                                   deserializer: self.deserializer,
                                   entities: self.entities,
                                   stats: self.stats,
                                   system_data: self.system_data,
                                   stat: stat,
                                   pd: self.pd,
                               })?
            .ok_or(DeError::invalid_length(1, &self))
    }
}

struct DeserializeUpdateOrRemove<'a, 'b: 'a, T: 'a, S: 'a> {
    deserializer: &'a mut T,
    entities: &'a mut Entities<'b>,
    stats: &'a mut WriteStorage<'b, NetStat>,
    system_data: &'a mut S,
    stat: NetStat,
    pd: PhantomData<&'b ()>,
}

const UPDATE_OR_REMOVE: &'static [&'static str;2] = &["Update", "Remove"];

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> DeserializeSeed<'de> for DeserializeUpdateOrRemove<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn deserialize<D>(self, deserializer: D) -> Result<Result<(), Error>, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_enum("Sync", UPDATE_OR_REMOVE, self)
    }
}


#[derive(Deserialize)]
enum UpdateOrRemove {
    Update,
    Remove,
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> Visitor<'de> for DeserializeUpdateOrRemove<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Seq of components sync-state")
    }

    fn visit_enum<A>(mut self, data: A) -> Result<Result<(), Error>, A::Error>
        where A: EnumAccess<'de>
    {
        use self::UpdateOrRemove::*;

        match data.variant()? {
            (Update, data) => {
                data.tuple_variant(T::COUNT,
                                   DeserializeComponents {
                                       deserializer: self.deserializer,
                                       entities: self.entities,
                                       stats: self.stats,
                                       system_data: self.system_data,
                                       stat: self.stat,
                                       pd: self.pd,
                                   })
            }
            (Remove, _) => Ok(self.deserializer.remove_entity(self.entities, self.stat)),
        }
    }
}

struct DeserializeComponents<'a, 'b: 'a, T: 'a, S: 'a> {
    deserializer: &'a mut T,
    entities: &'a mut Entities<'b>,
    stats: &'a mut WriteStorage<'b, NetStat>,
    system_data: &'a mut S,
    stat: NetStat,
    pd: PhantomData<&'b ()>,
}

impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> DeserializeSeed<'de> for DeserializeComponents<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn deserialize<D>(self, deserializer: D) -> Result<Result<(), Error>, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(self)
    }
}


impl<'de, 'a, 'b: 'a, T: 'a, S: 'a> Visitor<'de> for DeserializeComponents<'a, 'b, T, S>
    where S: SystemData<'b>,
          T: ComponentsDeserializer<'b, SystemData = S>
{
    type Value = Result<(), Error>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Seq of components sync-state")
    }

    fn visit_seq<A>(mut self, seq: A) -> Result<Result<(), Error>, A::Error>
        where A: SeqAccess<'de>
    {
        self.deserializer
               .deserialize_components(self.entities,
                                       self.stats,
                                       self.system_data,
                                       self.stat,
                                       seq)
    }
}
