
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, DeserializeSeed, Error as DeError, MapAccess,
                SeqAccess, Visitor};
use shred::Resource;
use specs::{Component, DenseVecStorage, Entity, World};

use net::{Error, ErrorKind, NetId, NetStat};
use net::sync::SyncSeq;


struct DeserializeResources<'a, T: 'a>(&'a mut World, &'a mut T);
impl<'de, 'a, T: 'a> DeserializeSeed<'de> for DeserializeResources<'a, T>
    where T: ResourcesDeserializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a, T: 'a> Visitor<'de> for DeserializeResources<'a, T>
    where T: ResourcesDeserializer
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Resources sync-state")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>
    {
        let sync_seq: SyncSeq =
            seq.next_element()?
                .ok_or(DeError::invalid_length(0, &self))?;
        self.1.sync_resources(self.0, sync_seq, seq)
    }
}


struct DeserializeEntities<'a, T: 'a>(&'a mut World, &'a mut T);
impl<'de, 'a, T: 'a> DeserializeSeed<'de> for DeserializeEntities<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a, T: 'a> Visitor<'de> for DeserializeEntities<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Sequence of `DeserializeEntity`")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>
    {
        while let Some(()) = seq.next_element_seed(DeserializeEntity(self.0, self.1))? {}
        Ok(())
    }
}

struct DeserializeEntity<'a, T: 'a>(&'a mut World, &'a mut T);
impl<'de, 'a, T: 'a> DeserializeSeed<'de> for DeserializeEntity<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_tuple_struct("DeserializeEntity", 2, self)
    }
}

impl<'de, 'a, T: 'a> Visitor<'de> for DeserializeEntity<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "NetId followed by components sync-state")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>
    {
        let stat: NetStat = seq.next_element()?
            .ok_or(DeError::invalid_length(0, &self))?;

        seq.next_element_seed(DeserializeComponents(self.0, self.1, stat))?
            .ok_or(DeError::invalid_length(1, &self))
    }
}

struct DeserializeComponents<'a, T: 'a>(&'a mut World, &'a mut T, NetStat);

impl<'de, 'a, T: 'a> DeserializeSeed<'de> for DeserializeComponents<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq(self)
    }
}


impl<'de, 'a, T: 'a> Visitor<'de> for DeserializeComponents<'a, T>
    where T: ComponentsDeserializer
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Seq of components sync-state")
    }

    fn visit_seq<A>(mut self, seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>
    {
        self.1.sync_components(self.0, self.2, seq)
    }
}


trait WorldDeserializer {
    fn sync<'de, 'a, A>(&'a mut self, world: &'a mut World, seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>;
}

trait ComponentsDeserializer {
    fn sync_components<'de, 'a, A>(&'a mut self,
                                   world: &'a mut World,
                                   stat: NetStat,
                                   seq: A)
                                   -> Result<(), A::Error>
        where A: SeqAccess<'de>;
}

trait ResourcesDeserializer {
    fn sync_resources<'de, 'a, A>(&'a mut self,
                                  world: &'a mut World,
                                  sync_seq: SyncSeq,
                                  seq: A)
                                  -> Result<(), A::Error>
        where A: SeqAccess<'de>;
}

struct BasicComponentsDeserializer<T> {
    ids: HashMap<NetId, Entity>,
    pd: PhantomData<T>,
}

struct BasicResourcesDeserializer<T> {
    sync_seq: SyncSeq,
    pd: PhantomData<T>,
}

macro_rules! impl_synchronizers {
    ($($a:ident,)*) => {
        impl<$($a,)*> ComponentsDeserializer for BasicComponentsDeserializer<($($a,)*)>
            where $($a: Component + for<'de> Deserialize<'de>,)*
        {
            fn sync_components<'de, 'a, SA>(&mut self,
                                            world: &'a mut World,
                                            stat: NetStat,
                                            mut seq: SA)
                                            -> Result<(), SA::Error>
                where SA: SeqAccess<'de>
            {
                use std::collections::hash_map::Entry::*;
                let (update, entity) = match self.ids.entry(stat.id) {
                    Occupied(entry) => {
                        let entity = *entry.get();
                        (world.write::<NetStat>()
                            .get_mut(entity).unwrap()
                            .sync_seq.update(stat.sync_seq), entity)
                    }
                    Vacant(entry) => {
                        (true, *entry.insert(world.create_entity().with(stat).build()))
                    }
                };

                if update {
                    let mut index = 0usize;
                    $(
                        let a: Option<$a> = seq.next_element()?
                            .ok_or(
                                DeError::invalid_length(index,
                                                        &"Sequence with all syncable components")
                            )?;
                        if let Some(a) = a {
                            world.write::<$a>().insert(entity, a);
                        } else {
                            world.write::<$a>().remove(entity);
                        }
                        index += 1;
                    )*;
                }
                Ok(())
            }
        }

        impl<$($a,)*> ResourcesDeserializer for BasicResourcesDeserializer<($($a,)*)>
            where $($a: Resource + for<'de> Deserialize<'de>,)*
        {
            fn sync_resources<'de, 'a, SA>(&'a mut self,
                                           world: &'a mut World,
                                           sync_seq: SyncSeq,
                                           mut seq: SA)
                                           -> Result<(), SA::Error>
                where SA: SeqAccess<'de>
            {
                if !self.sync_seq.update(sync_seq) {
                    return Ok(());
                }

                $(
                    let a: Option<$a> = seq.next_element()?;
                    if let Some(a) = a {
                        *world.write_resource::<$a>() = a;
                    }
                )*

                Ok(())
            }
        }
    };
}


impl_synchronizers!(A,);
impl_synchronizers!(A,B,);
impl_synchronizers!(A,B,C,);

struct BasicWorldDeserializer<R, C> {
    resources: R,
    components: C,
}

impl<R, C> WorldDeserializer for BasicWorldDeserializer<R, C>
    where R: ResourcesDeserializer,
          C: ComponentsDeserializer
{
    fn sync<'de, 'a, A>(&'a mut self, world: &'a mut World, mut seq: A) -> Result<(), A::Error>
        where A: SeqAccess<'de>
    {
        seq.next_element_seed(DeserializeResources(world, &mut self.resources))?
            .ok_or(DeError::invalid_length(0, &"(Resources, Components)"))?;
        seq.next_element_seed(DeserializeEntities(world, &mut self.components))?
            .ok_or(DeError::invalid_length(1, &"(Resources, Components)"))?;

        Ok(())
    }
}

